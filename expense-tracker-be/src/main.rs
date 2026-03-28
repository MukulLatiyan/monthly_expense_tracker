mod handlers;
mod models;
mod routes;
mod state;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use aws_config::BehaviorVersion;
use dotenv::dotenv;
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
    let table_name = env::var("DYNAMODB_TABLE")
        .unwrap_or_else(|_| "expense_tracker".to_string());
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());
    let aws_region = env::var("AWS_REGION").unwrap_or_else(|_| "ap-south-1".to_string());
    let use_local_dynamodb = env::var("DYNAMODB_LOCAL")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);

    println!("Initializing AWS SDK...");
    println!("AWS Region: {}", aws_region);
    println!("DynamoDB Table: {}", table_name);

    // Configure AWS SDK
    let mut config_builder = aws_config::defaults(BehaviorVersion::latest()).region(
        aws_config::Region::new(aws_region),
    );

    // If using local DynamoDB (for development), override the endpoint
    if use_local_dynamodb {
        let local_endpoint = env::var("DYNAMODB_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:8000".to_string());
        println!("Using local DynamoDB at: {}", local_endpoint);
        config_builder = config_builder.endpoint_url(local_endpoint);
    }

    let aws_config = config_builder.load().await;
    let client = aws_sdk_dynamodb::Client::new(&aws_config);

    // Create app state
    let app_state = web::Data::new(state::AppState::new(client, table_name.clone()));

    println!("Starting server at http://{}:{}", host, port);
    println!("Available endpoints:");
    println!("  GET    /debug");
    println!("  GET    /months/{{month}}/summary");
    println!("  GET    /months/{{month}}/expenses");
    println!("  POST   /months/{{month}}/expenses");
    println!("  PUT    /months/{{month}}/expenses/{{name}}/paid");
    println!("  PUT    /months/{{month}}/expenses/{{name}}/unpaid");
    println!("  PUT    /months/{{month}}/expenses/{{name}}/amount");
    println!("  DELETE /months/{{month}}/expenses/{{name}}");
    println!("  GET    /months/{{month}}/income");
    println!("  POST   /months/{{month}}/income");
    println!("  PUT    /months/{{month}}/income/{{name}}/received");
    println!("  PUT    /months/{{month}}/income/{{name}}/unreceived");
    println!("  PUT    /months/{{month}}/income/{{name}}/amount");
    println!("  DELETE /months/{{month}}/income/{{name}}");

    HttpServer::new(move || {
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
