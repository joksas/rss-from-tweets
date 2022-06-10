#[tokio::main]
async fn main() {
    server::run(3030).await;
}

mod server {
    use warp::Filter;
    pub async fn run(port: u16) {
        let routes = warp::path::end().and_then(handlers::root);
        warp::serve(routes).run(([127, 0, 0, 1], port)).await;
    }

    fn routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path::end().and_then(handlers::root)
    }

    mod handlers {
        use askama::Template;
        use warp::Rejection;

        pub async fn root() -> Result<impl warp::reply::Reply, Rejection> {
            #[derive(Template)]
            #[template(path = "root.html")]
            struct RootTemplate {}
            let tmpl = RootTemplate {};
            let output = match tmpl.render() {
                Ok(output) => output,
                Err(_e) => {
                    // TODO: Need custom rejection.
                    return Err(warp::reject::reject());
                }
            };
            Ok(warp::reply::html(output))
        }
    }

    #[cfg(test)]
    mod tests {
        use warp::http::StatusCode;
        use warp::test::request;

        use super::*;

        #[tokio::test]
        async fn test_root() {
            assert_status_code("/", StatusCode::OK).await;
            assert_status_code("/non-existent", StatusCode::NOT_FOUND).await;
        }

        async fn assert_status_code(path: &str, code: StatusCode) {
            let req = request().method("GET").path(path);
            let resp = req.reply(&routes()).await;

            assert_eq!(resp.status(), code);
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

            let user_id = id_from_username(username);
            match user_id.await {
                Ok(user_id) => assert_eq!(user_id, 12),
                Err(err) => panic!("{}", err),
            };
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
            let result = extract();
            match result {
                Ok(secrets) => assert_eq!(secrets.test_key, "test_value"),
                Err(err) => panic!("{}", err),
            }
        }
    }
}
