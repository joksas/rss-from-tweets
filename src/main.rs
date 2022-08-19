mod tmpl;
use actix_web::{get, App, HttpServer};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    const PORT: u16 = 8080;
    env_logger::init();

    match HttpServer::new(|| App::new().configure(handlers::config)).bind(("127.0.0.1", PORT)) {
        Ok(server) => {
            log::info!("Starting server at http://localhost:{}", PORT);
            server.run().await
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

mod handlers {
    use super::twitter;
    use actix_web::http::StatusCode;
    use actix_web::{get, web, HttpResponse};

    pub fn config(cfg: &mut web::ServiceConfig) {
        cfg.service(web::resource("/").route(web::get().to(root)));
        cfg.service(user_tweets);
        cfg.service(web::resource("/style.css").route(web::get().to(css)));
        cfg.default_service(web::route().to(not_found));
    }

    async fn root() -> HttpResponse {
        let output = super::tmpl::page(
            "Hello, World!",
            maud::html! {
                p { "How are you?" }
            },
        )
        .into_string();
        HttpResponse::Ok().content_type("text/html").body(output)
    }

    #[get("/users/{username}")]
    async fn user_tweets(username: web::Path<String>) -> HttpResponse {
        let user = twitter::user_by_username(&username).await;
        let user = match user {
            Ok(user) => user,
            Err(e) => {
                log::error!("Error retrieving user: {}", e);
                return HttpResponse::InternalServerError().body("Error");
            }
        };

        let tweets = twitter::user_tweets(&user, 5).await;
        let (data, expansions) = match tweets {
            Ok(tweets) => tweets,
            Err(e) => {
                log::error!("Error retrieving tweets: {}", e);
                return HttpResponse::InternalServerError().body("Error");
            }
        };

        let output = super::tmpl::user_tweets(&username, data, expansions).into_string();

        HttpResponse::Ok().content_type("text/html").body(output)
    }

    async fn not_found() -> HttpResponse {
        let output = super::tmpl::error_page(404, "Not Found").into_string();
        HttpResponse::NotFound()
            .content_type("text/html")
            .body(output)
    }

    async fn css() -> HttpResponse {
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
    use twitter_v2::query::MediaField;
    use twitter_v2::query::TweetExpansion;
    use twitter_v2::query::TweetField;
    use twitter_v2::{authorization, TwitterApi};

    pub async fn user_by_username(username: &str) -> Result<twitter_v2::User, String> {
        let secrets = secrets::extract()?;

        let auth = authorization::BearerToken::new(secrets.twitter.bearer_token);

        let user = match TwitterApi::new(auth)
            .get_user_by_username(username)
            .send()
            .await
        {
            Ok(user) => user,
            Err(err) => return Err(err.to_string()),
        };

        let user = match user.into_data() {
            Some(user) => user,
            None => return Err(String::from("User not found.")),
        };

        Ok(user)
    }

    pub async fn user_tweets(
        user: &twitter_v2::User,
        max_results: usize,
    ) -> Result<(Vec<twitter_v2::Tweet>, Vec<twitter_v2::Media>), String> {
        let secrets = secrets::extract()?;

        let auth = authorization::BearerToken::new(secrets.twitter.bearer_token);

        let tweets = match TwitterApi::new(auth)
            .get_user_tweets(user.id)
            .tweet_fields([TweetField::Entities, TweetField::Attachments])
            .media_fields([
                MediaField::MediaKey,
                MediaField::Url,
                MediaField::Type,
                MediaField::Width,
                MediaField::Height,
            ])
            .expansions([TweetExpansion::AttachmentsMediaKeys])
            .max_results(max_results)
            .send()
            .await
        {
            Ok(tweets) => tweets,
            Err(err) => return Err(err.to_string()),
        };
        let tweets_copy = tweets.clone();
        let tweets_data = match tweets.into_data() {
            Some(data) => data,
            None => return Err(String::from("Tweets not found.")),
        };
        let media_objects = match tweets_copy.into_includes() {
            Some(includes) => match includes.media {
                Some(media_objects) => media_objects,
                None => Vec::new(),
            },
            None => Vec::new(),
        };

        Ok((tweets_data, media_objects))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[tokio::test]
        async fn test_user_by_username() {
            let username = "jack";

            let user = user_by_username(username).await.unwrap();
            // See <https://web.archive.org/web/20220611133626/https://twitter.com/jack/status/49923786786615296>.
            assert_eq!(user.id, 12);
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
