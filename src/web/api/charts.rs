//! Chart Data API
//!
//! JSON endpoints for KPI visualization with ApexCharts.

use axum::{Json, extract::State};
use serde::Serialize;

use crate::db::AppState;

/// Generic chart data structure for ApexCharts.
#[derive(Serialize)]
pub struct ChartData {
    /// Labels for the chart (x-axis or series names).
    pub labels: Vec<String>,
    /// Numeric values corresponding to labels.
    pub values: Vec<i64>,
}

/// Time series data point.
#[derive(Serialize)]
pub struct TimeSeriesPoint {
    /// Timestamp in ISO format.
    pub x: String,
    /// Numeric value.
    pub y: i64,
}

/// Time series chart data.
#[derive(Serialize)]
pub struct TimeSeriesData {
    /// Series name.
    pub name: String,
    /// Data points.
    pub data: Vec<TimeSeriesPoint>,
}

/// GET /api/charts/assets-by-status
///
/// Returns asset count grouped by status for pie/donut chart.
pub async fn assets_by_status(State(state): State<AppState>) -> Json<ChartData> {
    #[derive(serde::Deserialize)]
    struct StatusCount {
        status: String,
        count: i64,
    }

    let counts: Vec<StatusCount> = state
        .db
        .query("SELECT status, count() as count FROM asset GROUP BY status")
        .await
        .ok()
        .and_then(|mut r| r.take(0).ok())
        .unwrap_or_default();

    Json(ChartData {
        labels: counts.iter().map(|c| format_status(&c.status)).collect(),
        values: counts.iter().map(|c| c.count).collect(),
    })
}

/// GET /api/charts/repairs-by-month
///
/// Returns repair operations count per month for the last 6 months.
pub async fn repairs_by_month(State(state): State<AppState>) -> Json<ChartData> {
    #[derive(serde::Deserialize)]
    struct MonthCount {
        month: String,
        count: i64,
    }

    let counts: Vec<MonthCount> = state
        .db
        .query(
            r#"
            SELECT 
                time::format(start_time, '%Y-%m') as month,
                count() as count 
            FROM repair_operation 
            WHERE start_time > time::now() - 6mo
            GROUP BY month
            ORDER BY month ASC
        "#,
        )
        .await
        .ok()
        .and_then(|mut r| r.take(0).ok())
        .unwrap_or_default();

    Json(ChartData {
        labels: counts.iter().map(|c| c.month.clone()).collect(),
        values: counts.iter().map(|c| c.count).collect(),
    })
}

/// GET /api/charts/employee-workload
///
/// Returns repair count per employee for bar chart.
pub async fn employee_workload(State(state): State<AppState>) -> Json<ChartData> {
    #[derive(serde::Deserialize)]
    struct EmployeeWorkload {
        name: String,
        count: i64,
    }

    let workloads: Vec<EmployeeWorkload> = state
        .db
        .query(
            r#"
            SELECT 
                employee.name as name,
                count() as count 
            FROM repair_operation
            WHERE status != 'Completed'
            GROUP BY employee_id
            FETCH employee
            ORDER BY count DESC
            LIMIT 10
        "#,
        )
        .await
        .ok()
        .and_then(|mut r| r.take(0).ok())
        .unwrap_or_default();

    Json(ChartData {
        labels: workloads.iter().map(|w| w.name.clone()).collect(),
        values: workloads.iter().map(|w| w.count).collect(),
    })
}

/// GET /api/charts/asset-value-by-category
///
/// Returns total asset value grouped by category.
pub async fn asset_value_by_category(State(state): State<AppState>) -> Json<ChartData> {
    #[derive(serde::Deserialize)]
    struct CategoryValue {
        category: String,
        total: i64,
    }

    let values: Vec<CategoryValue> = state
        .db
        .query(
            r#"
            SELECT 
                category,
                math::sum(value) as total 
            FROM asset 
            GROUP BY category
            ORDER BY total DESC
        "#,
        )
        .await
        .ok()
        .and_then(|mut r| r.take(0).ok())
        .unwrap_or_default();

    Json(ChartData {
        labels: values.iter().map(|v| v.category.clone()).collect(),
        values: values.iter().map(|v| v.total).collect(),
    })
}

/// Format status enum to user-friendly Arabic string.
fn format_status(status: &str) -> String {
    match status {
        "Available" => "متاح".to_string(),
        "InUse" => "قيد الاستخدام".to_string(),
        "Maintenance" => "صيانة".to_string(),
        "Retired" => "متقاعد".to_string(),
        _ => status.to_string(),
    }
}
