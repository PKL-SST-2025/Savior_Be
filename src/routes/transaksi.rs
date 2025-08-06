use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use uuid::Uuid;
use chrono::NaiveDate;
use serde::Deserialize;

use crate::database::Database;
use crate::models::transaksi::{Transaksi, TransaksiWithCategory, CreateTransaksiRequest, UpdateTransaksiRequest};

#[derive(Debug, Deserialize)]
pub struct TransaksiQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub kategori_id: Option<i32>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

// Get all transactions for a user
pub async fn get_user_transaksi(
    State(db): State<Database>,
    Path(user_id): Path<String>,
    Query(query): Query<TransaksiQuery>,
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

    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let mut sql = r#"
        SELECT 
            t.id,
            t.user_id::text as user_id,
            t.kategori_id,
            c.nama as kategori_nama,
            t.jumlah,
            t.deskripsi,
            t.tanggal,
            t.created_at,
            t.updated_at
        FROM transaksi t
        JOIN categories c ON t.kategori_id = c.id
        WHERE t.user_id = $1
    "#.to_string();

    let mut param_count = 2;
    
    // Add kategori filter if provided
    if query.kategori_id.is_some() {
        sql.push_str(&format!(" AND t.kategori_id = ${}", param_count));
        param_count += 1;
    }

    // Add date filters if provided
    if query.start_date.is_some() {
        sql.push_str(&format!(" AND t.tanggal >= ${}", param_count));
        param_count += 1;
    }

    if query.end_date.is_some() {
        sql.push_str(&format!(" AND t.tanggal <= ${}", param_count));
        param_count += 1;
    }

    sql.push_str(" ORDER BY t.tanggal DESC, t.created_at DESC");
    sql.push_str(&format!(" LIMIT ${} OFFSET ${}", param_count, param_count + 1));

    let mut query_builder = sqlx::query_as::<_, TransaksiWithCategory>(&sql)
        .bind(user_uuid)
        .bind(limit);

    if let Some(kategori_id) = query.kategori_id {
        query_builder = query_builder.bind(kategori_id);
    }

    if let Some(start_date) = query.start_date {
        if let Ok(date) = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d") {
            query_builder = query_builder.bind(date);
        }
    }

    if let Some(end_date) = query.end_date {
        if let Ok(date) = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d") {
            query_builder = query_builder.bind(date);
        }
    }

    query_builder = query_builder.bind(offset);

    let transaksi = query_builder
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
        "transaksi": transaksi
    })))
}

// Create new transaction for a user
pub async fn create_transaksi(
    State(db): State<Database>,
    Path(user_id): Path<String>,
    Json(payload): Json<CreateTransaksiRequest>,
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
    if payload.jumlah <= 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "message": "Jumlah harus lebih dari 0."
            }))
        ));
    }

    if payload.deskripsi.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "message": "Deskripsi tidak boleh kosong."
            }))
        ));
    }

    // Parse tanggal
    let tanggal = match NaiveDate::parse_from_str(&payload.tanggal, "%Y-%m-%d") {
        Ok(date) => date,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "status": "error",
                    "message": "Format tanggal tidak valid. Gunakan format YYYY-MM-DD."
                }))
            ));
        }
    };

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

    // Start transaction to update budget spent if exists
    let mut tx = db.begin().await.map_err(|err| {
        eprintln!("Transaction error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": "Terjadi kesalahan pada server."
            }))
        )
    })?;

    // Insert transaksi baru
    let new_transaksi = sqlx::query_as::<_, Transaksi>(
        "INSERT INTO transaksi (user_id, kategori_id, jumlah, deskripsi, tanggal) VALUES ($1, $2, $3, $4, $5) RETURNING *"
    )
    .bind(user_uuid)
    .bind(payload.kategori_id)
    .bind(payload.jumlah)
    .bind(&payload.deskripsi.trim())
    .bind(tanggal)
    .fetch_one(&mut *tx)
    .await
    .map_err(|err| {
        eprintln!("Database error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": "Gagal membuat transaksi."
            }))
        )
    })?;

    // Update budget spent if exists for this user and category
    sqlx::query(
        "UPDATE budgets SET spent = COALESCE(spent, 0) + $1, updated_at = NOW() WHERE user_id = $2 AND kategori_id = $3"
    )
    .bind(payload.jumlah)
    .bind(user_uuid)
    .bind(payload.kategori_id)
    .execute(&mut *tx)
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

    // Commit transaction
    tx.commit().await.map_err(|err| {
        eprintln!("Transaction commit error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": "Gagal menyimpan transaksi."
            }))
        )
    })?;

    // Response sukses
    Ok(Json(json!({
        "status": "success",
        "message": "Transaksi berhasil dibuat!",
        "data": new_transaksi
    })))
}

// Update transaction
pub async fn update_transaksi(
    State(db): State<Database>,
    Path((user_id, transaksi_id)): Path<(String, i32)>,
    Json(payload): Json<UpdateTransaksiRequest>,
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

    // Cek apakah transaksi exists dan belongs to user
    let existing_transaksi = sqlx::query_as::<_, Transaksi>(
        "SELECT * FROM transaksi WHERE id = $1 AND user_id = $2"
    )
    .bind(transaksi_id)
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

    if existing_transaksi.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "error",
                "message": "Transaksi tidak ditemukan."
            }))
        ));
    }

    let old_transaksi = existing_transaksi.unwrap();

    // Parse tanggal if provided
    let tanggal = if let Some(tanggal_str) = &payload.tanggal {
        Some(match NaiveDate::parse_from_str(tanggal_str, "%Y-%m-%d") {
            Ok(date) => date,
            Err(_) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "status": "error",
                        "message": "Format tanggal tidak valid. Gunakan format YYYY-MM-DD."
                    }))
                ));
            }
        })
    } else {
        None
    };

    // Validasi kategori if provided
    if let Some(kategori_id) = payload.kategori_id {
        let category_exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM categories WHERE id = $1)")
            .bind(kategori_id)
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
    }

    // Start transaction to update budget spent
    let mut tx = db.begin().await.map_err(|err| {
        eprintln!("Transaction error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": "Terjadi kesalahan pada server."
            }))
        )
    })?;

    // Update transaksi
    let updated_transaksi = sqlx::query_as::<_, Transaksi>(
        r#"UPDATE transaksi SET 
           kategori_id = COALESCE($1, kategori_id),
           jumlah = COALESCE($2, jumlah),
           deskripsi = COALESCE($3, deskripsi),
           tanggal = COALESCE($4, tanggal),
           updated_at = NOW() 
           WHERE id = $5 RETURNING *"#
    )
    .bind(payload.kategori_id)
    .bind(payload.jumlah)
    .bind(payload.deskripsi.as_ref().map(|s| s.trim()))
    .bind(tanggal)
    .bind(transaksi_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|err| {
        eprintln!("Database error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": "Gagal mengupdate transaksi."
            }))
        )
    })?;

    // Update budget spent - subtract old amount and add new amount
    let jumlah_diff = updated_transaksi.jumlah - old_transaksi.jumlah;
    
    // If category changed, update both old and new category budgets
    if let Some(new_kategori_id) = payload.kategori_id {
        if new_kategori_id != old_transaksi.kategori_id {
            // Subtract from old category budget
            sqlx::query(
                "UPDATE budgets SET spent = GREATEST(COALESCE(spent, 0) - $1, 0), updated_at = NOW() WHERE user_id = $2 AND kategori_id = $3"
            )
            .bind(old_transaksi.jumlah)
            .bind(user_uuid)
            .bind(old_transaksi.kategori_id)
            .execute(&mut *tx)
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

            // Add to new category budget
            sqlx::query(
                "UPDATE budgets SET spent = COALESCE(spent, 0) + $1, updated_at = NOW() WHERE user_id = $2 AND kategori_id = $3"
            )
            .bind(updated_transaksi.jumlah)
            .bind(user_uuid)
            .bind(new_kategori_id)
            .execute(&mut *tx)
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
        } else {
            // Same category, just update the difference
            sqlx::query(
                "UPDATE budgets SET spent = COALESCE(spent, 0) + $1, updated_at = NOW() WHERE user_id = $2 AND kategori_id = $3"
            )
            .bind(jumlah_diff)
            .bind(user_uuid)
            .bind(old_transaksi.kategori_id)
            .execute(&mut *tx)
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
        }
    } else {
        // Category not changed, just update the amount difference
        sqlx::query(
            "UPDATE budgets SET spent = COALESCE(spent, 0) + $1, updated_at = NOW() WHERE user_id = $2 AND kategori_id = $3"
        )
        .bind(jumlah_diff)
        .bind(user_uuid)
        .bind(old_transaksi.kategori_id)
        .execute(&mut *tx)
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
    }

    // Commit transaction
    tx.commit().await.map_err(|err| {
        eprintln!("Transaction commit error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": "Gagal menyimpan perubahan."
            }))
        )
    })?;

    // Response sukses
    Ok(Json(json!({
        "status": "success",
        "message": "Transaksi berhasil diupdate!",
        "data": updated_transaksi
    })))
}

// Delete transaction
pub async fn delete_transaksi(
    State(db): State<Database>,
    Path((user_id, transaksi_id)): Path<(String, i32)>,
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

    // Cek apakah transaksi exists dan belongs to user
    let existing_transaksi = sqlx::query_as::<_, Transaksi>(
        "SELECT * FROM transaksi WHERE id = $1 AND user_id = $2"
    )
    .bind(transaksi_id)
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

    if existing_transaksi.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "error",
                "message": "Transaksi tidak ditemukan."
            }))
        ));
    }

    let transaksi = existing_transaksi.unwrap();

    // Start transaction to update budget spent
    let mut tx = db.begin().await.map_err(|err| {
        eprintln!("Transaction error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": "Terjadi kesalahan pada server."
            }))
        )
    })?;

    // Delete transaksi
    sqlx::query("DELETE FROM transaksi WHERE id = $1")
        .bind(transaksi_id)
        .execute(&mut *tx)
        .await
        .map_err(|err| {
            eprintln!("Database error: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": "Gagal menghapus transaksi."
                }))
            )
        })?;

    // Update budget spent - subtract the deleted transaction amount
    sqlx::query(
        "UPDATE budgets SET spent = GREATEST(COALESCE(spent, 0) - $1, 0), updated_at = NOW() WHERE user_id = $2 AND kategori_id = $3"
    )
    .bind(transaksi.jumlah)
    .bind(user_uuid)
    .bind(transaksi.kategori_id)
    .execute(&mut *tx)
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

    // Commit transaction
    tx.commit().await.map_err(|err| {
        eprintln!("Transaction commit error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": "Gagal menyimpan perubahan."
            }))
        )
    })?;

    // Response sukses
    Ok(Json(json!({
        "status": "success",
        "message": "Transaksi berhasil dihapus!"
    })))
}

// Get transaction by ID
pub async fn get_transaksi_by_id(
    State(db): State<Database>,
    Path((user_id, transaksi_id)): Path<(String, i32)>,
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

    let transaksi = sqlx::query_as::<_, TransaksiWithCategory>(
        r#"
        SELECT 
            t.id,
            t.user_id::text as user_id,
            t.kategori_id,
            c.nama as kategori_nama,
            t.jumlah,
            t.deskripsi,
            t.tanggal,
            t.created_at,
            t.updated_at
        FROM transaksi t
        JOIN categories c ON t.kategori_id = c.id
        WHERE t.id = $1 AND t.user_id = $2
        "#
    )
    .bind(transaksi_id)
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

    match transaksi {
        Some(transaksi) => Ok(Json(json!({
            "status": "success",
            "data": transaksi
        }))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "error",
                "message": "Transaksi tidak ditemukan."
            }))
        ))
    }
}
