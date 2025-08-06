use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::database::Database;
use crate::models::budget::{Budget, BudgetWithCategory, CreateBudgetRequest, UpdateBudgetRequest};

// Get all budgets for a user
pub async fn get_user_budgets(
    State(db): State<Database>,
    Path(user_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Parse user_id as UUID
    let user_uuid = match Uuid::parse_str(&user_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "status": "error",
                    "message": "Invalid user ID format."
                }))
            ));
        }
    };

    let budgets = sqlx::query_as::<_, BudgetWithCategory>(
        r#"
        SELECT 
            b.id,
            b.user_id::text as user_id,
            b.kategori_id,
            c.nama as kategori_nama,
            b.amount,
            COALESCE(b.spent, 0) as spent,
            CASE 
                WHEN b.amount > 0 THEN (COALESCE(b.spent, 0)::float / b.amount::float * 100.0)
                ELSE 0.0
            END as percentage
        FROM budgets b
        JOIN categories c ON b.kategori_id = c.id
        WHERE b.user_id = $1
        ORDER BY b.created_at DESC
        "#
    )
    .bind(user_uuid)
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

    Ok(Json(json!({
        "status": "success",
        "budgets": budgets
    })))
}

// Create new budget for a user
pub async fn create_budget(
    State(db): State<Database>,
    Path(user_id): Path<String>,
    Json(payload): Json<CreateBudgetRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Parse user_id as UUID
    let user_uuid = match Uuid::parse_str(&user_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "status": "error",
                    "message": "Invalid user ID format."
                }))
            ));
        }
    };

    // Validasi input
    if payload.amount <= 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "message": "Amount harus lebih dari 0."
            }))
        ));
    }

    // Cek apakah kategori exists
    let category_exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM categories WHERE id = $1)")
        .bind(payload.kategori_id)
        .fetch_one(&db)
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

    if !category_exists {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "message": "Kategori tidak ditemukan."
            }))
        ));
    }

    // Cek apakah user sudah punya budget untuk kategori ini
    let existing_budget = sqlx::query_as::<_, Budget>(
        "SELECT * FROM budgets WHERE user_id = $1 AND kategori_id = $2"
    )
    .bind(user_uuid)
    .bind(payload.kategori_id)
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

    if existing_budget.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({
                "status": "error",
                "message": "Budget untuk kategori ini sudah ada."
            }))
        ));
    }

    // Insert budget baru
    let new_budget = sqlx::query_as::<_, Budget>(
        "INSERT INTO budgets (user_id, kategori_id, amount) VALUES ($1, $2, $3) RETURNING *"
    )
    .bind(user_uuid)
    .bind(payload.kategori_id)
    .bind(payload.amount)
    .fetch_one(&db)
    .await
    .map_err(|err| {
        eprintln!("Database error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": "Gagal membuat budget."
            }))
        )
    })?;

    // Response sukses
    Ok(Json(json!({
        "status": "success",
        "message": "Budget berhasil dibuat!",
        "data": new_budget
    })))
}

// Update budget
pub async fn update_budget(
    State(db): State<Database>,
    Path((user_id, budget_id)): Path<(String, i32)>,
    Json(payload): Json<UpdateBudgetRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Parse user_id as UUID
    let user_uuid = match Uuid::parse_str(&user_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "status": "error",
                    "message": "Invalid user ID format."
                }))
            ));
        }
    };

    // Cek apakah budget exists dan belongs to user
    let existing_budget = sqlx::query_as::<_, Budget>(
        "SELECT * FROM budgets WHERE id = $1 AND user_id = $2"
    )
    .bind(budget_id)
    .bind(user_uuid)
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

    if existing_budget.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "error",
                "message": "Budget tidak ditemukan."
            }))
        ));
    }

    let budget = existing_budget.unwrap();

    // Update budget
    let updated_budget = sqlx::query_as::<_, Budget>(
        "UPDATE budgets SET amount = COALESCE($1, amount), spent = COALESCE($2, spent), updated_at = NOW() WHERE id = $3 RETURNING *"
    )
    .bind(payload.amount)
    .bind(payload.spent)
    .bind(budget_id)
    .fetch_one(&db)
    .await
    .map_err(|err| {
        eprintln!("Database error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": "Gagal mengupdate budget."
            }))
        )
    })?;

    // Response sukses
    Ok(Json(json!({
        "status": "success",
        "message": "Budget berhasil diupdate!",
        "data": updated_budget
    })))
}

// Delete budget
pub async fn delete_budget(
    State(db): State<Database>,
    Path((user_id, budget_id)): Path<(String, i32)>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Parse user_id as UUID
    let user_uuid = match Uuid::parse_str(&user_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "status": "error",
                    "message": "Invalid user ID format."
                }))
            ));
        }
    };

    // Cek apakah budget exists dan belongs to user
    let existing_budget = sqlx::query_as::<_, Budget>(
        "SELECT * FROM budgets WHERE id = $1 AND user_id = $2"
    )
    .bind(budget_id)
    .bind(user_uuid)
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

    if existing_budget.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "error",
                "message": "Budget tidak ditemukan."
            }))
        ));
    }

    // Delete budget
    sqlx::query("DELETE FROM budgets WHERE id = $1")
        .bind(budget_id)
        .execute(&db)
        .await
        .map_err(|err| {
            eprintln!("Database error: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": "Gagal menghapus budget."
                }))
            )
        })?;

    // Response sukses
    Ok(Json(json!({
        "status": "success",
        "message": "Budget berhasil dihapus!"
    })))
}

// Get budget by ID
pub async fn get_budget_by_id(
    State(db): State<Database>,
    Path((user_id, budget_id)): Path<(String, i32)>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Parse user_id as UUID
    let user_uuid = match Uuid::parse_str(&user_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "status": "error",
                    "message": "Invalid user ID format."
                }))
            ));
        }
    };

    let budget = sqlx::query_as::<_, BudgetWithCategory>(
        r#"
        SELECT 
            b.id,
            b.user_id::text as user_id,
            b.kategori_id,
            c.nama as kategori_nama,
            b.amount,
            COALESCE(b.spent, 0) as spent,
            CASE 
                WHEN b.amount > 0 THEN (COALESCE(b.spent, 0)::float / b.amount::float * 100.0)
                ELSE 0.0
            END as percentage
        FROM budgets b
        JOIN categories c ON b.kategori_id = c.id
        WHERE b.id = $1 AND b.user_id = $2
        "#
    )
    .bind(budget_id)
    .bind(user_uuid)
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

    match budget {
        Some(budget) => Ok(Json(json!({
            "status": "success",
            "data": budget
        }))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "error",
                "message": "Budget tidak ditemukan."
            }))
        ))
    }
}
