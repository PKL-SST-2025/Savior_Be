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
    pub tertinggi_hari_ini: Option<i64>,
    pub terendah_hari_ini: Option<i64>,
    pub tertinggi_bulan_ini: Option<i64>,
    pub terendah_bulan_ini: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct DashboardResponse {
    pub total_bulan_ini: i64,
    pub total_hari_ini: i64,
    pub tertinggi_bulan_ini: i64,
    pub tertinggi_hari_ini: i64,
    pub terendah_bulan_ini: i64,
    pub terendah_hari_ini: i64,
    pub pengeluaran_mingguan: Vec<ChartDataPoint>,
    pub transaksi_terakhir: Vec<TransaksiTerakhir>,
}

#[derive(Debug, Serialize)]
pub struct ChartDataPoint {
    pub hari: String,
    pub jumlah: i64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct TransaksiTerakhir {
    pub id: i32,
    pub deskripsi: String,
    pub jumlah: i32,  // ✅ FIXED: Use i32 to match database INT4
    pub tanggal: String,
    pub kategori_nama: String,
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
    pub year: Option<i32>,
    pub month: Option<u32>,
}
