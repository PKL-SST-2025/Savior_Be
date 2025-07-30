use axum::{
    routing::{get, post, put, delete},
    Router,
    http::StatusCode,
};
use dotenvy::dotenv;
use std::env;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::cors::{Any, CorsLayer};

mod database;
mod models;
mod routes;

use routes::auth::{signup, signin};

use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() {
    // Load environment dari .env file
    dotenv().ok();

    // Ambil DATABASE_URL dari environment
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL tidak ditemukan di .env");

    // Inisialisasi koneksi pool ke database
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Gagal menghubungkan ke database PostgreSQL");

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await
        .expect("Failed to run migrations");

    // Static file untuk frontend SolidJS (dari folder ../fe/dist)
    let serve_dir = ServeDir::new("../fe/dist")
        .not_found_service(ServeFile::new("../fe/dist/index.html"));

    // Middleware CORS untuk izinkan request dari frontend
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Error handler
    async fn handle_404() -> StatusCode {
        StatusCode::NOT_FOUND
    }

    // Definisikan semua route backend API
    let app = Router::new()
        // Auth routes
        .route("/signup", post(signup))
        .route("/signin", post(signin))

        // Test route
        .route("/hello", get(|| async { "Hello from Axum!" }))

        // 404 handler
        .fallback(handle_404)

        // Inject state pool DB dan middleware CORS
        .with_state(pool)
        .layer(cors)

        // Serve frontend fallback (pindah ke akhir)
        .fallback_service(serve_dir);

    // Jalankan server di localhost:3000
    let addr = "127.0.0.1:3000";
    println!("🚀 Server running at http://{}", addr);
    println!("✅ Database connected and migrations completed");
    println!("🔗 Endpoints available at http://{}", addr);

    // Binding listener
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    // Serve aplikasi
    axum::serve(listener, app).await.unwrap();
}
