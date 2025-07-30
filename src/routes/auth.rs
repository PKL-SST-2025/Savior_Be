use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::database::Database;
use crate::models::user::{User, SignupRequest};

#[derive(Debug, serde::Deserialize)]
pub struct SigninRequest {
    pub email: String,
    pub password: String,
}

pub async fn signup(
    State(db): State<Database>,
    Json(payload): Json<SignupRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Validasi input
    if payload.email.is_empty() || payload.password.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "message": "Email dan password wajib diisi."
            }))
        ));
    }

    // Cek apakah email sudah terdaftar
    let existing_user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_optional(&db)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": "Terjadi kesalahan pada server."
                }))
            )
        })?;

    if existing_user.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({
                "status": "error",
                "message": "Email sudah terdaftar."
            }))
        ));
    }

    // Generate user ID
    let user_id = Uuid::new_v4();

    // Note: Dalam production, Anda harus hash password menggunakan bcrypt atau argon2
    // Untuk sementara, kita simpan password mentah (TIDAK AMAN untuk production!)
    let password_hash = payload.password; // TODO: Hash password properly

    // Insert user baru
    let new_user = sqlx::query_as::<_, User>(
        "INSERT INTO users (id, username, email, password_hash) VALUES ($1, $2, $3, $4) RETURNING *"
    )
    .bind(user_id)
    .bind(&payload.email) // Menggunakan email sebagai username sementara
    .bind(&payload.email)
    .bind(&password_hash)
    .fetch_one(&db)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": "Gagal membuat akun."
            }))
        )
    })?;

    // Response sukses
    Ok(Json(json!({
        "status": "success",
        "message": "Akun berhasil dibuat!",
        "user": {
            "id": new_user.id,
            "email": new_user.email,
            "created_at": new_user.created_at
        }
    })))
}

pub async fn signin(
    State(db): State<Database>,
    Json(payload): Json<SigninRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Validasi input
    if payload.email.is_empty() || payload.password.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "message": "Email dan password wajib diisi."
            }))
        ));
    }

    // Cari user berdasarkan email
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_optional(&db)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": "Terjadi kesalahan pada server."
                }))
            )
        })?;

    // Cek apakah user ditemukan
    let user = match user {
        Some(user) => user,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "status": "error",
                    "message": "Email atau password salah."
                }))
            ));
        }
    };

    // Verifikasi password
    // Note: Dalam production, gunakan bcrypt::verify untuk hash password
    if user.password_hash != payload.password {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "status": "error",
                "message": "Email atau password salah."
            }))
        ));
    }

    // Response sukses login
    Ok(Json(json!({
        "status": "success",
        "message": "Login berhasil!",
        "user_id": user.id,
        "user": {
            "id": user.id,
            "email": user.email,
            "username": user.username,
            "created_at": user.created_at
        }
    })))
}
