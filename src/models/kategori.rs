use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Kategori {
    pub id: i32,
    pub nama: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateKategoriRequest {
    pub nama: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateKategoriRequest {
    pub nama: String,
}
