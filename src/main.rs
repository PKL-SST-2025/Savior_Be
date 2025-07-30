use axum::{
    http::StatusCode,
    routing::get,
    Router,
};
use tower_http::services::{ServeDir, ServeFile};

mod database;
mod models;
mod routes;

use database::{create_database_connection, run_migrations};
use routes::user::user_routes;

#[tokio::main]
async fn main() {
    // Initialize database connection
    let db = create_database_connection()
        .await
        .expect("Failed to connect to database");

    // Run migrations
    run_migrations(&db)
        .await
        .expect("Failed to run migrations");

    // API routes
    let api_routes = Router::new()
        // Merge user routes
        .merge(user_routes())
        
        // Hello endpoint for testing
        .route("/hello", get(|| async { "Hello from your Axum backend!" }))
        .with_state(db);

    // This service handles serving static files from the ../fe/dist directory.
    // The not_found_service is the key to making a Single Page App (SPA) work:
    // if a file is not found (e.g., /about, /users/123), it serves index.html.
    let serve_dir = ServeDir::new("../fe/dist")
        .not_found_service(ServeFile::new("../fe/dist/index.html"));

    let app = Router::new()
        // API routes should come first
        .nest("/api", api_routes)
        
        // This makes the static file service handle all other requests
        .fallback_service(serve_dir);

    let addr = "127.0.0.1:3000";
    println!("🚀 Listening on http://{}", addr);
    println!("📊 Database connected and migrations completed");
    println!("🔗 API endpoints available at http://{}/api", addr);

    // Create the TCP listener
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap();
    
    // This is the correct way to run the server in Axum 0.7
    axum::serve(listener, app).await.unwrap();
}
