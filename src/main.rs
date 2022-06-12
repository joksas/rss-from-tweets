use actix_web::{App, HttpServer};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().configure(handlers::config))
        .bind(("127.0.0.1", 3030))?
        .run()
        .await
}

mod handlers {
    use actix_web::http::StatusCode;
    use actix_web::{web, HttpResponse};
    use askama::Template;

    pub fn config(cfg: &mut web::ServiceConfig) {
        cfg.service(web::resource("/").route(web::get().to(root)));
        cfg.service(web::resource("/style.css").route(web::get().to(css)));
        cfg.default_service(web::route().to(not_found));
    }

    async fn root() -> HttpResponse {
        #[derive(Template)]
        #[template(path = "root.html")]
        struct RootTemplate {}
        let tmpl = RootTemplate {};
        match tmpl.render() {
            Ok(output) => return HttpResponse::Ok().content_type("text/html").body(output),
            Err(_e) => HttpResponse::InternalServerError().body("Internal Server Error"),
        }
    }

    async fn not_found() -> HttpResponse {
        #[derive(Template)]
        #[template(path = "error.html")]
        struct ErrorTemplate {
            message: String,
            code: u16,
        }
        let tmpl = ErrorTemplate {
            code: 404,
            message: "Not Found".to_string(),
        };
        match tmpl.render() {
            Ok(output) => {
                return HttpResponse::NotFound()
                    .content_type("text/html")
                    .body(output)
            }
            Err(_e) => HttpResponse::InternalServerError().body("Internal Server Error"),
        }
    }

    pub async fn css() -> HttpResponse {
        HttpResponse::build(StatusCode::OK)
            .content_type("text/css")
            .body(include_str!("../assets/style.css"))
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use actix_web::http::StatusCode;
        use actix_web::{test, App};

        #[actix_web::test]
        async fn test_status_codes() {
            let app = test::init_service(App::new().configure(config)).await;

            let req = test::TestRequest::get().uri("/").to_request();
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), StatusCode::OK);

            let req = test::TestRequest::get().uri("/non-existent").to_request();
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        }
    }
}

mod twitter {
    use super::secrets;
    use chrono::prelude::*;
    use reqwest::Client;

    fn url_tweets(user_id: u32, start_time: DateTime<Utc>) -> String {
        format!(
            "https://api.twitter.com/2/users/{user_id}/tweets?start_time={datetime_iso_8601}",
            user_id = user_id,
            datetime_iso_8601 = start_time.to_rfc3339_opts(SecondsFormat::Millis, true),
        )
    }

    fn url_id_from_username(username: &str) -> String {
        format!(
            "https://api.twitter.com/2/users/by/username/{username}",
            username = username
        )
    }

    async fn id_from_username(username: &str) -> Result<u64, String> {
        let client = Client::new();

        let url = url_id_from_username(username);
        let secrets = secrets::extract();
        let secrets = match secrets {
            Ok(secrets) => secrets,
            Err(err) => return Err(err),
        };
        println!("URL: {}", url);

        let request = client
            .get(url)
            .bearer_auth(secrets.twitter.bearer_token)
            .build();
        let request = match request {
            Ok(request) => request,
            Err(e) => return Err(e.to_string()),
        };

        let response = client.execute(request).await;
        let response = match response {
            Ok(response) => response,
            Err(e) => return Err(e.to_string()),
        };

        let response = match response.json::<serde_json::Value>().await {
            Ok(response) => response,
            Err(e) => return Err(e.to_string()),
        };

        let string_user_id = match response["data"]["id"].as_str() {
            Some(user_id) => user_id,
            None => return Err("Could not parse user_id".to_string()),
        };

        let user_id = match string_user_id.parse::<u64>() {
            Ok(user_id) => user_id,
            Err(e) => return Err(e.to_string()),
        };

        Ok(user_id)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_url_tweets() {
            let user_id = 123;
            let start_time = Utc.ymd(2022, 6, 27).and_hms(1, 30, 0);

            let url = url_tweets(user_id, start_time);
            assert_eq!(
                url,
                "https://api.twitter.com/2/users/123/tweets?start_time=2022-06-27T01:30:00.000Z"
            );
        }

        #[test]
        fn test_url_id_from_username() {
            let user_id = "jack";

            let url = url_id_from_username(user_id);
            assert_eq!(url, "https://api.twitter.com/2/users/by/username/jack");
        }

        #[tokio::test]
        async fn test_id_from_username() {
            let username = "jack";

            let user_id = id_from_username(username).await.unwrap();
            // See <https://web.archive.org/web/20220611133626/https://twitter.com/jack/status/49923786786615296>.
            assert_eq!(user_id, 12);
        }
    }
}

mod secrets {
    use serde::Deserialize;
    use std::process::Command;

    #[derive(Deserialize)]
    pub struct TwitterSecrets {
        api_key: String,
        api_key_secret: String,
        pub bearer_token: String,
    }

    #[derive(Deserialize)]
    pub struct Secrets {
        pub twitter: TwitterSecrets,
        test_key: String,
    }

    pub fn extract() -> Result<Secrets, String> {
        let output = Command::new("sops")
            .arg("-d")
            .arg("--output-type")
            .arg("json")
            .arg("src/secrets.yaml")
            .output()
            .expect("failed to execute process");

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).into_owned());
        }

        let secrets = match serde_json::from_slice(&output.stdout) {
            Ok(secrets) => secrets,
            Err(err) => return Err(format!("Parsing output error: {}", err)),
        };

        Ok(secrets)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_extract() {
            let secrets = extract().unwrap();
            assert_eq!(secrets.test_key, "test_value");
        }
    }
}
