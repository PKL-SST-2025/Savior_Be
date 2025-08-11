use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};

use crate::database::Database;
use crate::models::kategori::{Kategori, CreateKategoriRequest, UpdateKategoriRequest};

// Get all categories
pub async fn get_all_kategori(
    State(db): State<Database>,
) -> Result<Json<Vec<Kategori>>, (StatusCode, Json<Value>)> {
    let categories = sqlx::query_as::<_, Kategori>("SELECT * FROM categories ORDER BY created_at DESC")
        .fetch_all(&db)
        .await
        .map_err(|err| {
            eprintln!("Database error: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": "Terjadi kesalahan pada server."
                }))
            )
        })?;

    Ok(Json(categories))
}

// Create new category
pub async fn create_kategori(
    State(db): State<Database>,
    Json(payload): Json<CreateKategoriRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Validasi input
    if payload.nama.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "message": "Nama kategori wajib diisi."
            }))
        ));
    }

    // Cek apakah kategori dengan nama yang sama sudah ada
    let existing_category = sqlx::query_as::<_, Kategori>("SELECT * FROM categories WHERE nama = $1")
        .bind(&payload.nama.trim())
        .fetch_optional(&db)
        .await
        .map_err(|err| {
            eprintln!("Database error: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": "Terjadi kesalahan pada server."
                }))
            )
        })?;

    if existing_category.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({
                "status": "error",
                "message": "Kategori dengan nama tersebut sudah ada."
            }))
        ));
    }

    // Insert kategori baru
    let new_category = sqlx::query_as::<_, Kategori>(
        "INSERT INTO categories (nama) VALUES ($1) RETURNING *"
    )
    .bind(&payload.nama.trim())
    .fetch_one(&db)
    .await
    .map_err(|err| {
        eprintln!("Database error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": "Gagal membuat kategori."
            }))
        )
    })?;

    // Response sukses
    Ok(Json(json!({
        "status": "success",
        "message": "Kategori berhasil dibuat!",
        "data": new_category
    })))
}

// Update category
pub async fn update_kategori(
    State(db): State<Database>,
    Path(kategori_id): Path<i32>,
    Json(payload): Json<UpdateKategoriRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Validasi input
    if payload.nama.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "message": "Nama kategori wajib diisi."
            }))
        ));
    }

    // Cek apakah kategori dengan ID tersebut ada
    let existing_category = sqlx::query_as::<_, Kategori>("SELECT * FROM categories WHERE id = $1")
        .bind(kategori_id)
        .fetch_optional(&db)
        .await
        .map_err(|err| {
            eprintln!("Database error: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": "Terjadi kesalahan pada server."
                }))
            )
        })?;

    if existing_category.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "error",
                "message": "Kategori tidak ditemukan."
            }))
        ));
    }

    // Cek apakah ada kategori lain dengan nama yang sama
    let duplicate_category = sqlx::query_as::<_, Kategori>("SELECT * FROM categories WHERE nama = $1 AND id != $2")
        .bind(&payload.nama.trim())
        .bind(kategori_id)
        .fetch_optional(&db)
        .await
        .map_err(|err| {
            eprintln!("Database error: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": "Terjadi kesalahan pada server."
                }))
            )
        })?;

    if duplicate_category.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({
                "status": "error",
                "message": "Kategori dengan nama tersebut sudah ada."
            }))
        ));
    }

    // Update kategori
    let updated_category = sqlx::query_as::<_, Kategori>(
        "UPDATE categories SET nama = $1, updated_at = NOW() WHERE id = $2 RETURNING *"
    )
    .bind(&payload.nama.trim())
    .bind(kategori_id)
    .fetch_one(&db)
    .await
    .map_err(|err| {
        eprintln!("Database error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": "Gagal mengupdate kategori."
            }))
        )
    })?;

    // Response sukses
    Ok(Json(json!({
        "status": "success",
        "message": "Kategori berhasil diupdate!",
        "data": updated_category
    })))
}

// Delete category
pub async fn delete_kategori(
    State(db): State<Database>,
    Path(kategori_id): Path<i32>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Cek apakah kategori dengan ID tersebut ada
    let existing_category = sqlx::query_as::<_, Kategori>("SELECT * FROM categories WHERE id = $1")
        .bind(kategori_id)
        .fetch_optional(&db)
        .await
        .map_err(|err| {
            eprintln!("Database error: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": "Terjadi kesalahan pada server."
                }))
            )
        })?;

    if existing_category.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "error",
                "message": "Kategori tidak ditemukan."
            }))
        ));
    }

    // Delete kategori
    sqlx::query("DELETE FROM categories WHERE id = $1")
        .bind(kategori_id)
        .execute(&db)
        .await
        .map_err(|err| {
            eprintln!("Database error: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": "Gagal menghapus kategori."
                }))
            )
        })?;

    // Response sukses
    Ok(Json(json!({
        "status": "success",
        "message": "Kategori berhasil dihapus!"
    })))
}

// Get category by ID
pub async fn get_kategori_by_id(
    State(db): State<Database>,
    Path(kategori_id): Path<i32>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let category = sqlx::query_as::<_, Kategori>("SELECT * FROM categories WHERE id = $1")
        .bind(kategori_id)
        .fetch_optional(&db)
        .await
        .map_err(|err| {
            eprintln!("Database error: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": "Terjadi kesalahan pada server."
                }))
            )
        })?;

    match category {
        Some(category) => Ok(Json(json!({
            "status": "success",
            "data": category
        }))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "error",
                "message": "Kategori tidak ditemukan."
            }))
        ))
    }
}
