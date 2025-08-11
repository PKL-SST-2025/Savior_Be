use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::database::Database;
use crate::models::{Post, CreatePost, UpdatePost};

pub async fn get_posts(State(db): State<Database>) -> Result<Json<Value>, StatusCode> {
    let posts = sqlx::query_as::<_, Post>(
        "SELECT p.*, u.username as author_username 
         FROM posts p 
         JOIN users u ON p.author_id = u.id 
         ORDER BY p.created_at DESC"
    )
    .fetch_all(&db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({
        "status": "success",
        "data": posts
    })))
}

pub async fn get_post_by_id(
    State(db): State<Database>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    let post = sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE id = $1")
        .bind(id)
        .fetch_optional(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match post {
        Some(post) => Ok(Json(json!({
            "status": "success",
            "data": post
        }))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn create_post(
    State(db): State<Database>,
    Json(payload): Json<CreatePost>,
) -> Result<Json<Value>, StatusCode> {
    let post_id = Uuid::new_v4();
    
    let post = sqlx::query_as::<_, Post>(
        "INSERT INTO posts (id, title, content, author_id) VALUES ($1, $2, $3, $4) RETURNING *"
    )
    .bind(post_id)
    .bind(&payload.title)
    .bind(&payload.content)
    .bind(&payload.author_id)
    .fetch_one(&db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({
        "status": "success",
        "data": post
    })))
}

pub async fn update_post(
    State(db): State<Database>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdatePost>,
) -> Result<Json<Value>, StatusCode> {
    let post = sqlx::query_as::<_, Post>(
        "UPDATE posts SET 
         title = COALESCE($1, title),
         content = COALESCE($2, content),
         updated_at = NOW()
         WHERE id = $3 
         RETURNING *"
    )
    .bind(&payload.title)
    .bind(&payload.content)
    .bind(id)
    .fetch_optional(&db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match post {
        Some(post) => Ok(Json(json!({
            "status": "success",
            "data": post
        }))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn delete_post(
    State(db): State<Database>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    let result = sqlx::query("DELETE FROM posts WHERE id = $1")
        .bind(id)
        .execute(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Json(json!({
        "status": "success",
        "message": "Post deleted successfully"
    })))
}

pub async fn get_posts_by_user(
    State(db): State<Database>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    let posts = sqlx::query_as::<_, Post>(
        "SELECT * FROM posts WHERE author_id = $1 ORDER BY created_at DESC"
    )
    .bind(user_id)
    .fetch_all(&db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({
        "status": "success",
        "data": posts
    })))
}
