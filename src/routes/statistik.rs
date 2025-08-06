use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use uuid::Uuid;
use chrono::{NaiveDate, Local, Datelike};

use crate::database::Database;
use crate::models::statistik::{StatistikResponse, PengeluaranKategori, RingkasanPengeluaran, PengeluaranRange, StatistikQuery};

// Get user statistics
pub async fn get_user_statistik(
    State(db): State<Database>,
    Path(user_id): Path<String>,
    Query(query): Query<StatistikQuery>,
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

    // Determine date range based on filter
    let (start_date, end_date) = match query.filter.as_deref() {
        Some("daily") => {
            let today = Local::now().naive_local().date();
            (today, today)
        },
        Some("weekly") => {
            let today = Local::now().naive_local().date();
            let start = today - chrono::Duration::days(7);
            (start, today)
        },
        Some("monthly") => {
            let today = Local::now().naive_local().date();
            let start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
            (start, today)
        },
        _ => {
            // Default: current month
            let today = Local::now().naive_local().date();
            let start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
            (start, today)
        }
    };

    // Override with custom dates if provided
    let final_start_date = if let Some(custom_start) = query.start_date {
        match NaiveDate::parse_from_str(&custom_start, "%Y-%m-%d") {
            Ok(date) => date,
            Err(_) => start_date,
        }
    } else {
        start_date
    };

    let final_end_date = if let Some(custom_end) = query.end_date {
        match NaiveDate::parse_from_str(&custom_end, "%Y-%m-%d") {
            Ok(date) => date,
            Err(_) => end_date,
        }
    } else {
        end_date
    };

    // Get total pengeluaran for percentage calculation
    let total_pengeluaran: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(jumlah), 0) FROM transaksi WHERE user_id = $1 AND tanggal >= $2 AND tanggal <= $3"
    )
    .bind(user_uuid)
    .bind(final_start_date)
    .bind(final_end_date)
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

    // Get pengeluaran per kategori
    let pengeluaran_per_kategori: Vec<PengeluaranKategori> = if total_pengeluaran > 0 {
        sqlx::query_as::<_, PengeluaranKategori>(
            r#"
            SELECT 
                c.nama as kategori_nama,
                COALESCE(SUM(t.jumlah), 0) as total_pengeluaran,
                CASE 
                    WHEN $4 > 0 THEN ROUND((COALESCE(SUM(t.jumlah), 0) * 100.0 / $4), 2)
                    ELSE 0
                END as persentase
            FROM categories c
            LEFT JOIN transaksi t ON c.id = t.kategori_id 
                AND t.user_id = $1 
                AND t.tanggal >= $2 
                AND t.tanggal <= $3
            GROUP BY c.id, c.nama
            HAVING COALESCE(SUM(t.jumlah), 0) > 0
            ORDER BY total_pengeluaran DESC
            "#
        )
        .bind(user_uuid)
        .bind(final_start_date)
        .bind(final_end_date)
        .bind(total_pengeluaran)
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
        })?
    } else {
        Vec::new()
    };

    // Get total transaksi count
    let total_transaksi: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM transaksi WHERE user_id = $1 AND tanggal >= $2 AND tanggal <= $3"
    )
    .bind(user_uuid)
    .bind(final_start_date)
    .bind(final_end_date)
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

    // Calculate rata-rata harian
    let days_diff = (final_end_date - final_start_date).num_days() + 1;
    let rata_rata_harian = if days_diff > 0 {
        total_pengeluaran as f64 / days_diff as f64
    } else {
        0.0
    };

    let ringkasan = RingkasanPengeluaran {
        total_pengeluaran,
        rata_rata_harian,
        total_transaksi,
    };

    let statistik = StatistikResponse {
        pengeluaran_per_kategori,
        ringkasan,
    };

    Ok(Json(json!({
        "status": "success",
        "data": statistik,
        "filter_applied": {
            "start_date": final_start_date.format("%Y-%m-%d").to_string(),
            "end_date": final_end_date.format("%Y-%m-%d").to_string(),
            "filter_type": query.filter.unwrap_or_else(|| "monthly".to_string())
        }
    })))
}

// Get global spending range statistics (for the donut chart)
pub async fn get_spending_ranges() -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // This is demo data for the spending ranges
    // In real implementation, you would calculate this from all users' data
    let spending_ranges = vec![
        PengeluaranRange {
            range_label: "$ 0 - $ 20,000".to_string(),
            jumlah_user: 20,
            persentase: 20.0,
        },
        PengeluaranRange {
            range_label: "$ 20,000 - $ 30,000".to_string(),
            jumlah_user: 25,
            persentase: 25.0,
        },
        PengeluaranRange {
            range_label: "$ 30,000 - $ 60,000".to_string(),
            jumlah_user: 40,
            persentase: 40.0,
        },
        PengeluaranRange {
            range_label: "more than $ 60,000".to_string(),
            jumlah_user: 15,
            persentase: 15.0,
        },
    ];

    Ok(Json(json!({
        "status": "success",
        "data": spending_ranges
    })))
}

// Get user monthly spending for range categorization
pub async fn get_user_monthly_spending(
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

    // Get current month spending
    let today = Local::now().naive_local().date();
    let start_of_month = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
    
    let monthly_spending: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(jumlah), 0) FROM transaksi WHERE user_id = $1 AND tanggal >= $2 AND tanggal <= $3"
    )
    .bind(user_uuid)
    .bind(start_of_month)
    .bind(today)
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

    // Categorize spending range
    let spending_category = if monthly_spending <= 20000 {
        "$ 0 - $ 20,000"
    } else if monthly_spending <= 30000 {
        "$ 20,000 - $ 30,000"
    } else if monthly_spending <= 60000 {
        "$ 30,000 - $ 60,000"
    } else {
        "more than $ 60,000"
    };

    Ok(Json(json!({
        "status": "success",
        "data": {
            "monthly_spending": monthly_spending,
            "spending_category": spending_category,
            "month": today.format("%Y-%m").to_string()
        }
    })))
}
