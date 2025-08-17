use axum::{
    routing::{get, post, put, delete},
    Router,
    http::StatusCode,
};
use dotenvy::dotenv;
use std::env;
use tower_http::{
    services::{ServeDir, ServeFile},
    cors::{Any, CorsLayer},
};
use sqlx::postgres::PgPoolOptions;

mod database;
mod models;
mod routes;

use routes::auth::{signup, signin, forgot_password};
use routes::user::get_user_by_id;
use routes::profile::{get_profile, update_profile, update_email, update_password};
use routes::kategori::{get_all_kategori, create_kategori, update_kategori, delete_kategori, get_kategori_by_id};
use routes::budget::{get_user_budgets, create_budget, update_budget, delete_budget, get_budget_by_id};
use routes::transaksi::{get_user_transaksi, create_transaksi, update_transaksi, delete_transaksi, get_transaksi_by_id};
use routes::statistik::{get_user_statistik, get_spending_ranges, get_user_monthly_spending, get_dashboard_data};

#[tokio::main]
async fn main() {
    // 1️⃣ Load variabel environment
    dotenv().ok();

    // 2️⃣ Ambil DATABASE_URL
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL tidak ditemukan di .env");

    // 3️⃣ Inisialisasi koneksi pool PostgreSQL
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Gagal menghubungkan ke database PostgreSQL");

    // 4️⃣ Jalankan migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Gagal menjalankan migrations");

    // 5️⃣ Static file frontend
    let serve_dir = ServeDir::new("../fe/dist")
        .not_found_service(ServeFile::new("../fe/dist/index.html"));

    // 6️⃣ Middleware CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 7️⃣ Handler 404
    async fn handle_404() -> StatusCode {
        StatusCode::NOT_FOUND
    }

    // 8️⃣ Definisi routes API
    let api_routes = Router::new()
        // Auth
        .route("/signup", post(signup))
        .route("/signin", post(signin))
        .route("/forgot-password", post(forgot_password))

        // User
        .route("/api/user/:user_id", get(get_user_by_id))

        // Profile
        .route("/api/profile/:user_id", get(get_profile))
        .route("/api/profile/:user_id", put(update_profile))
        .route("/api/profile/:user_id/email", put(update_email))
        .route("/api/profile/:user_id/password", put(update_password))

        // Kategori
        .route("/api/kategori", get(get_all_kategori))
        .route("/api/kategori", post(create_kategori))
        .route("/api/kategori/:id", get(get_kategori_by_id))
        .route("/api/kategori/:id", put(update_kategori))
        .route("/api/kategori/:id", delete(delete_kategori))

        // Budget
        .route("/api/budget/:user_id", get(get_user_budgets))
        .route("/api/budget/:user_id", post(create_budget))
        .route("/api/budget/:user_id/:budget_id", get(get_budget_by_id))
        .route("/api/budget/:user_id/:budget_id", put(update_budget))
        .route("/api/budget/:user_id/:budget_id", delete(delete_budget))

        // Transaksi
        .route("/api/transaksi/:user_id", get(get_user_transaksi))
        .route("/api/transaksi/:user_id", post(create_transaksi))
        .route("/api/transaksi/:user_id/:transaksi_id", get(get_transaksi_by_id))
        .route("/api/transaksi/:user_id/:transaksi_id", put(update_transaksi))
        .route("/api/transaksi/:user_id/:transaksi_id", delete(delete_transaksi))

        // Statistik
        .route("/api/statistik/ranges", get(get_spending_ranges))
        .route("/api/statistik/:user_id", get(get_user_statistik))
        .route("/api/statistik/:user_id/monthly", get(get_user_monthly_spending))
        .route("/api/dashboard/:user_id", get(get_dashboard_data))

        // Test route
        .route("/hello", get(|| async { "Hello from Axum!" }));

    // 9️⃣ Gabungkan API + middleware
    let app = Router::new()
        .merge(api_routes)
        .with_state(pool)
        .layer(cors)
        .fallback(handle_404) // Handler 404 API
        .fallback_service(serve_dir); // Fallback ke frontend

    // 🔟 Jalankan server
    let addr = "0.0.0.0:3000";
    println!("🚀 Server running at http://{}", addr);
    println!("✅ Database connected and migrations completed");
    println!("🔗 Endpoints available at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
