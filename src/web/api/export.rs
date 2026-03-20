//! Data Export API — CSV export for admin list views
//!
//! يتيح تصدير بيانات الجداول كملفات CSV.

use axum::{
    extract::{Query, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Deserialize;

use crate::db::AppState;

/// Allowed tables for export (prevents arbitrary table access)
const EXPORTABLE_TABLES: &[(&str, &str)] = &[
    ("employee", "employees"),
    ("client", "clients"),
    ("asset", "assets"),
    ("machine", "machines"),
    ("invoice", "invoices"),
    ("project", "projects"),
    ("repair_operation", "repairs"),
    ("trainee", "trainees"),
    ("certificate", "certificates"),
    ("organization", "organizations"),
    ("event", "events"),
    ("leave_request", "leave_requests"),
];

#[derive(Deserialize)]
pub struct ExportParams {
    /// Table name to export
    pub table: String,
}

/// GET /api/export?table=employee — Export table data as CSV
pub async fn export_csv(
    State(state): State<AppState>,
    Query(params): Query<ExportParams>,
) -> Response {
    let table = params.table.trim();

    // Validate table name against allowlist
    let filename = match EXPORTABLE_TABLES.iter().find(|(t, _)| *t == table) {
        Some((_, name)) => format!("{}.csv", name),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                format!("Table '{}' is not exportable", table),
            )
                .into_response();
        }
    };

    // Query all records from the table as JSON
    let query = format!("SELECT * FROM {} ORDER BY created_at DESC", table);
    let result: Result<Vec<serde_json::Value>, _> = state
        .db
        .query(&query)
        .await
        .and_then(|mut r| r.take(0));

    let records = match result {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Export query failed for table {}: {:?}", table, e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Export failed").into_response();
        }
    };

    if records.is_empty() {
        return (StatusCode::OK, "No data to export").into_response();
    }

    // Build CSV from JSON records
    let csv_content = json_to_csv(&records);

    // Add UTF-8 BOM for Excel compatibility with Arabic text
    let mut output = Vec::new();
    output.extend_from_slice(&[0xEF, 0xBB, 0xBF]); // UTF-8 BOM
    output.extend_from_slice(csv_content.as_bytes());

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/csv; charset=utf-8")
        .header(
            header::CONTENT_DISPOSITION,
            HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename))
                .unwrap_or_else(|_| HeaderValue::from_static("attachment; filename=\"export.csv\"")),
        )
        .body(axum::body::Body::from(output))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

/// Convert a vector of JSON objects to CSV string.
fn json_to_csv(records: &[serde_json::Value]) -> String {
    // Collect all unique keys from all records for headers
    let mut headers: Vec<String> = Vec::new();
    for record in records {
        if let serde_json::Value::Object(map) = record {
            for key in map.keys() {
                // Skip internal fields
                if key == "password_hash" || key == "api_key_hash" {
                    continue;
                }
                if !headers.contains(key) {
                    headers.push(key.clone());
                }
            }
        }
    }

    // Sort headers for consistency (id first, then alphabetical)
    headers.sort_by(|a, b| {
        if a == "id" {
            std::cmp::Ordering::Less
        } else if b == "id" {
            std::cmp::Ordering::Greater
        } else if a == "name" || a == "title" {
            std::cmp::Ordering::Less
        } else if b == "name" || b == "title" {
            std::cmp::Ordering::Greater
        } else {
            a.cmp(b)
        }
    });

    let mut csv = String::new();

    // Header row
    csv.push_str(&headers.iter().map(|h| escape_csv(h)).collect::<Vec<_>>().join(","));
    csv.push('\n');

    // Data rows
    for record in records {
        let row: Vec<String> = headers
            .iter()
            .map(|key| {
                if let serde_json::Value::Object(map) = record {
                    match map.get(key) {
                        Some(serde_json::Value::String(s)) => escape_csv(s),
                        Some(serde_json::Value::Number(n)) => n.to_string(),
                        Some(serde_json::Value::Bool(b)) => b.to_string(),
                        Some(serde_json::Value::Null) | None => String::new(),
                        Some(v) => escape_csv(&v.to_string()),
                    }
                } else {
                    String::new()
                }
            })
            .collect();
        csv.push_str(&row.join(","));
        csv.push('\n');
    }

    csv
}

/// Escape a CSV field value (wrap in quotes if needed).
fn escape_csv(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}
