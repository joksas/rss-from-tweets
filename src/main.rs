mod handlers;
mod secrets;
mod tmpl;
mod twitter;
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
