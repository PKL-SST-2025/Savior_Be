use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Transaksi {
    pub id: i32,
    pub user_id: Uuid,
    pub kategori_id: i32,
    pub jumlah: i32,
    pub deskripsi: String,
    pub tanggal: NaiveDate,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TransaksiWithCategory {
    pub id: i32,
    pub user_id: String,
    pub kategori_id: i32,
    pub kategori_nama: String,
    pub jumlah: i32,
    pub deskripsi: String,
    pub tanggal: NaiveDate,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTransaksiRequest {
    pub kategori_id: i32,
    pub jumlah: i32,
    pub deskripsi: String,
    pub tanggal: String, // Format: "YYYY-MM-DD"
}

#[derive(Debug, Deserialize)]
pub struct UpdateTransaksiRequest {
    pub kategori_id: Option<i32>,
    pub jumlah: Option<i32>,
    pub deskripsi: Option<String>,
    pub tanggal: Option<String>, // Format: "YYYY-MM-DD"
}
