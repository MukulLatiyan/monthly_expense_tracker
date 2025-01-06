mod handlers;
mod models;
mod routes;
mod state;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use dotenv::dotenv;
use std::collections::HashMap;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load .env file
    dotenv().ok();

    // Set up logging
    env::set_var(
        "RUST_LOG",
        env::var("RUST_LOG").unwrap_or_else(|_| "debug".to_string()),
    );
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    // Get configuration from environment
    let file_path = env::var("DATA_FILE_PATH").expect("DATA_FILE_PATH must be set");
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());

    println!("Loading data from: {}", file_path);

    if let Ok(metadata) = std::fs::metadata(&file_path) {
        println!("File exists, size: {} bytes", metadata.len());
    } else {
        println!("File doesn't exist, creating new file");
        std::fs::write(&file_path, "{}")?;
    }

    let data = match state::AppState::load_data(file_path.as_str()) {
        Ok(data) => {
            println!("Successfully loaded data: {:?}", data);
            data
        }
        Err(e) => {
            println!("Error loading data: {}, using empty HashMap", e);
            HashMap::new()
        }
    };

    let app_state = web::Data::new(state::AppState {
        data: std::sync::Mutex::new(data),
        data_file: file_path,
    });

    println!("Starting server at http://{}:{}", host, port);

    HttpServer::new(move || {
        println!("Configuring server...");
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .wrap(Logger::new("%r %s %b %{Referer}i %a %T"))
            .app_data(app_state.clone())
            .configure(routes::configure_routes)
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
