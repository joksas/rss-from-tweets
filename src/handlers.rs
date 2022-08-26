use crate::tmpl;
use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(root)));
    cfg.service(user_tweets);
    cfg.service(web::resource("/style.css").route(web::get().to(css)));
    cfg.default_service(web::route().to(not_found));
}

async fn root() -> HttpResponse {
    let output = tmpl::page(
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
    let user = super::twitter::user_by_username(&username).await;
    let user = match user {
        Ok(user) => user,
        Err(e) => {
            log::error!("Error retrieving user: {}", e);
            return HttpResponse::InternalServerError().body("Error");
        }
    };

    let tweets = super::twitter::user_tweets(&user, 5).await;
    let (tweet_data, referenced_tweet_data, expansions) = match tweets {
        Ok(tweets) => tweets,
        Err(e) => {
            log::error!("Error retrieving tweets: {}", e);
            return HttpResponse::InternalServerError().body("Error");
        }
    };

    let output =
        tmpl::user_tweets(&username, tweet_data, referenced_tweet_data, expansions).into_string();

    HttpResponse::Ok().content_type("text/html").body(output)
}

async fn not_found() -> HttpResponse {
    let output = tmpl::error_page(404, "Not Found").into_string();
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
