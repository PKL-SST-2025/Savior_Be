use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct PengeluaranKategori {
    pub kategori_nama: String,
    pub total_pengeluaran: i64,
    pub persentase: f64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct RingkasanPengeluaran {
    pub total_pengeluaran: i64,
    pub rata_rata_harian: f64,
    pub total_transaksi: i64,
}

#[derive(Debug, Serialize)]
pub struct StatistikResponse {
    pub pengeluaran_per_kategori: Vec<PengeluaranKategori>,
    pub ringkasan: RingkasanPengeluaran,
}

#[derive(Debug, Serialize, FromRow)]
pub struct PengeluaranRange {
    pub range_label: String,
    pub jumlah_user: i64,
    pub persentase: f64,
}

#[derive(Debug, Deserialize)]
pub struct StatistikQuery {
    pub filter: Option<String>, // "daily", "weekly", "monthly"
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}
