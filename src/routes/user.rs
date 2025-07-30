use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::database::Database;
use crate::models::user::{User, CreateUser, UpdateUser};

pub async fn get_users(State(db): State<Database>) -> Result<Json<Value>, StatusCode> {
    let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
        .fetch_all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({
        "status": "success",
        "data": users
    })))
}

pub async fn get_user_by_id(
    State(db): State<Database>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match user {
        Some(user) => Ok(Json(json!({
            "status": "success",
            "data": user
        }))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn create_user(
    State(db): State<Database>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<Value>, StatusCode> {
    // Note: In production, you should hash the password before storing
    let user_id = Uuid::new_v4();
    
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (id, username, email, password_hash) VALUES ($1, $2, $3, $4) RETURNING *"
    )
    .bind(user_id)
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&payload.password) // In production, hash this password!
    .fetch_one(&db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({
        "status": "success",
        "data": user
    })))
}

pub async fn update_user(
    State(db): State<Database>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateUser>,
) -> Result<Json<Value>, StatusCode> {
    let user = sqlx::query_as::<_, User>(
        "UPDATE users SET 
         username = COALESCE($1, username),
         email = COALESCE($2, email),
         updated_at = NOW()
         WHERE id = $3 
         RETURNING *"
    )
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(id)
    .fetch_optional(&db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match user {
        Some(user) => Ok(Json(json!({
            "status": "success",
            "data": user
        }))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn delete_user(
    State(db): State<Database>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Json(json!({
        "status": "success",
        "message": "User deleted successfully"
    })))
}
