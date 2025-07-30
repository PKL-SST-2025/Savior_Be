use axum::{
    http::StatusCode,
    routing::get,
    Router,
};
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() {
    // This service handles serving static files from the ../fe/dist directory.
    // The not_found_service is the key to making a Single Page App (SPA) work:
    // if a file is not found (e.g., /about, /users/123), it serves index.html.
    let serve_dir = ServeDir::new("../fe/dist")
        .not_found_service(ServeFile::new("../fe/dist/index.html"));

    let app = Router::new()
        // Your API routes should come first
        .route("/api/hello", get(|| async { "Hello from your Axum backend!" }))
        
        // This makes the static file service handle all other requests
        .fallback_service(serve_dir);

    let addr = "127.0.0.1:3000";
    println!("🚀 Listening on http://{}", addr);

    // Create the TCP listener
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap();
    
    // This is the correct way to run the server in Axum 0.7
    axum::serve(listener, app).await.unwrap();
}