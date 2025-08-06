use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::database::Database;
use crate::models::user::User;
use crate::models::profile::{Profile, UpdateProfileRequest, UpdateEmailRequest, UpdatePasswordRequest};

pub async fn get_profile(
    State(db): State<Database>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Cari user berdasarkan ID untuk mendapatkan data profile
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&db)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "message": "Terjadi kesalahan pada server."
                }))
            )
        })?;

    let user = match user {
        Some(user) => user,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "success": false,
                    "message": "User tidak ditemukan."
                }))
            ));
        }
    };

    // Response sukses dengan data profile
    Ok(Json(json!({
        "success": true,
        "message": "Profile berhasil dimuat.",
        "profile": {
            "id": user.id,
            "first_name": user.username.split_whitespace().next().unwrap_or(""),
            "last_name": user.username.split_whitespace().skip(1).collect::<Vec<&str>>().join(" "),
            "email": user.email,
            "created_at": user.created_at,
            "updated_at": user.updated_at
        }
    })))
}

pub async fn update_profile(
    State(db): State<Database>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Validasi input
    if payload.first_name.is_none() && payload.last_name.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "message": "Tidak ada data yang diupdate."
            }))
        ));
    }

    // Gabungkan first_name dan last_name menjadi username
    let full_name = format!(
        "{} {}",
        payload.first_name.as_deref().unwrap_or(""),
        payload.last_name.as_deref().unwrap_or("")
    ).trim().to_string();

    // Update username di database
    let updated_user = sqlx::query_as::<_, User>(
        "UPDATE users SET username = $1, updated_at = NOW() WHERE id = $2 RETURNING *"
    )
    .bind(&full_name)
    .bind(user_id)
    .fetch_optional(&db)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "message": "Gagal mengupdate profile."
            }))
        )
    })?;

    match updated_user {
        Some(user) => Ok(Json(json!({
            "success": true,
            "message": "Profile berhasil diupdate!",
            "profile": {
                "id": user.id,
                "first_name": payload.first_name,
                "last_name": payload.last_name,
                "email": user.email,
                "updated_at": user.updated_at
            }
        }))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "message": "User tidak ditemukan."
            }))
        ))
    }
}

pub async fn update_email(
    State(db): State<Database>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<UpdateEmailRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Validasi input
    if payload.new_email.is_empty() || payload.password.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "message": "Email dan password wajib diisi."
            }))
        ));
    }

    // Cari user dan verifikasi password
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&db)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "message": "Terjadi kesalahan pada server."
                }))
            )
        })?;

    let user = match user {
        Some(user) => user,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "success": false,
                    "message": "User tidak ditemukan."
                }))
            ));
        }
    };

    // Verifikasi password
    if user.password_hash != payload.password {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "success": false,
                "message": "Password salah."
            }))
        ));
    }

    // Cek apakah email sudah digunakan user lain
    let existing_user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1 AND id != $2")
        .bind(&payload.new_email)
        .bind(user_id)
        .fetch_optional(&db)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "message": "Terjadi kesalahan pada server."
                }))
            )
        })?;

    if existing_user.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({
                "success": false,
                "message": "Email sudah digunakan oleh user lain."
            }))
        ));
    }

    // Update email
    let updated_user = sqlx::query_as::<_, User>(
        "UPDATE users SET email = $1, updated_at = NOW() WHERE id = $2 RETURNING *"
    )
    .bind(&payload.new_email)
    .bind(user_id)
    .fetch_one(&db)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "message": "Gagal mengupdate email."
            }))
        )
    })?;

    // Response sukses
    Ok(Json(json!({
        "success": true,
        "message": "Email berhasil diupdate!",
        "profile": {
            "id": updated_user.id,
            "email": updated_user.email,
            "updated_at": updated_user.updated_at
        }
    })))
}

pub async fn update_password(
    State(db): State<Database>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<UpdatePasswordRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Validasi input
    if payload.current_password.is_empty() || payload.new_password.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "message": "Password lama dan baru wajib diisi."
            }))
        ));
    }

    // Validasi panjang password baru
    if payload.new_password.len() < 6 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "message": "Password baru minimal 6 karakter."
            }))
        ));
    }

    // Cari user dan verifikasi password lama
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&db)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "message": "Terjadi kesalahan pada server."
                }))
            )
        })?;

    let user = match user {
        Some(user) => user,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "success": false,
                    "message": "User tidak ditemukan."
                }))
            ));
        }
    };

    // Verifikasi password lama
    if user.password_hash != payload.current_password {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "success": false,
                "message": "Password lama salah."
            }))
        ));
    }

    // Update password
    // Note: Dalam production, hash password menggunakan bcrypt atau argon2
    let new_password_hash = payload.new_password; // TODO: Hash password properly

    let updated_user = sqlx::query_as::<_, User>(
        "UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2 RETURNING *"
    )
    .bind(&new_password_hash)
    .bind(user_id)
    .fetch_one(&db)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "message": "Gagal mengupdate password."
            }))
        )
    })?;

    // Response sukses
    Ok(Json(json!({
        "success": true,
        "message": "Password berhasil diupdate!",
        "profile": {
            "id": updated_user.id,
            "updated_at": updated_user.updated_at
        }
    })))
}
