use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Budget {
    pub id: i32,
    pub user_id: Uuid,
    pub kategori_id: i32,
    pub amount: i32,
    pub spent: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BudgetWithCategory {
    pub id: i32,
    pub user_id: String,
    pub kategori_id: i32,
    pub kategori_nama: String,
    pub amount: i32,
    pub spent: i32,
    pub percentage: f64,
}

#[derive(Debug, Deserialize)]
pub struct CreateBudgetRequest {
    pub kategori_id: i32,
    pub amount: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBudgetRequest {
    pub amount: Option<i32>,
    pub spent: Option<i32>,
}
