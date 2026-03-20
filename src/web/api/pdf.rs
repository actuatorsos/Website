//! PDF Generation API
//!
//! توليد تقارير PDF والشهادات

use std::io::BufWriter;

use axum::{
    Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use chrono::prelude::*;
use printpdf::*;

use crate::db::AppState;

// ============================================================================
// Helpers
// ============================================================================

fn load_font(doc: &PdfDocumentReference) -> Result<IndirectFontRef, String> {
    // Load Arabic font
    let font_path = "static/fonts/Alexandria-Regular.ttf";
    let font_file =
        std::fs::File::open(font_path).map_err(|e| format!("Failed to open font file: {}", e))?;

    doc.add_external_font(font_file)
        .map_err(|e| format!("Failed to load font: {}", e))
}

// ============================================================================
// Attendance Report
// ============================================================================

#[derive(serde::Deserialize)]
struct ReportParams {
    date: Option<String>,
}

async fn generate_attendance_report(
    State(state): State<AppState>,
    Query(params): Query<ReportParams>,
) -> Result<Response, StatusCode> {
    let target_date = params
        .date
        .unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string());

    // Fetch data
    let attendance_records: Vec<crate::domains::hr::models::Attendance> =
        crate::domains::hr::repository::get_attendance_by_date(&state, &target_date)
            .await
            .unwrap_or_default();

    // Create PDF
    let (doct, page1, layer1) = PdfDocument::new(
        format!("Attendance Report - {}", target_date),
        Mm(210.0),
        Mm(297.0),
        "Layer 1",
    );

    let font = load_font(&doct).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let current_layer = doct.get_page(page1).get_layer(layer1);

    // Title
    current_layer.use_text(
        "Dr. Machine - Attendance Report",
        24.0,
        Mm(10.0),
        Mm(280.0),
        &font,
    );
    current_layer.use_text(
        format!("Date: {}", target_date),
        14.0,
        Mm(10.0),
        Mm(270.0),
        &font,
    );

    // Table Header
    let y_start = 250.0;
    let mut y = y_start;
    let row_height = 10.0;

    current_layer.use_text("Name", 12.0, Mm(10.0), Mm(y), &font);
    current_layer.use_text("Type", 12.0, Mm(80.0), Mm(y), &font);
    current_layer.use_text("Time", 12.0, Mm(120.0), Mm(y), &font);
    current_layer.use_text("Work Hours", 12.0, Mm(160.0), Mm(y), &font);

    y -= row_height;

    // Table Rows
    for record in attendance_records {
        if y < 20.0 {
            break;
        }

        let name = record.person_name.clone().unwrap_or_default();
        let p_type = format!("{:?}", record.person_type);

        // Parse dates
        let check_in_dt = DateTime::parse_from_rfc3339(&record.check_in).ok();
        let check_out_dt = record
            .check_out
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok());

        let time = check_in_dt
            .map(|dt| dt.format("%H:%M").to_string())
            .unwrap_or_else(|| "-".to_string());

        let hours = if let (Some(out), Some(in_dt)) = (check_out_dt, check_in_dt) {
            let duration = out.signed_duration_since(in_dt);
            format!("{:.1}h", duration.num_minutes() as f64 / 60.0)
        } else {
            "-".to_string()
        };

        current_layer.use_text(name, 10.0, Mm(10.0), Mm(y), &font);
        current_layer.use_text(p_type, 10.0, Mm(80.0), Mm(y), &font);
        current_layer.use_text(time, 10.0, Mm(120.0), Mm(y), &font);
        current_layer.use_text(hours, 10.0, Mm(160.0), Mm(y), &font);

        y -= row_height;
    }

    // Save to buffer
    let mut buffer = Vec::new();
    {
        let mut writer = BufWriter::new(&mut buffer);
        doct.save(&mut writer)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Serve as download
    let filename = format!("attendance_{}.pdf", target_date);
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/pdf".parse().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", filename)
            .parse()
            .unwrap(),
    );

    Ok((headers, buffer).into_response())
}

// ============================================================================
// Certificate PDF
// ============================================================================

async fn generate_certificate_pdf(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, StatusCode> {
    let cert: crate::domains::finance::models::Certificate =
        crate::domains::finance::repository::get_certificate_by_credential_id(&state, &id)
            .await
            .map_err(|_| StatusCode::NOT_FOUND)?;

    let (doct, page1, layer1) = PdfDocument::new("Certificate", Mm(297.0), Mm(210.0), "Layer 1");
    let current_layer = doct.get_page(page1).get_layer(layer1);

    let font = load_font(&doct).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    current_layer.use_text(
        "Certificate of Completion",
        40.0,
        Mm(100.0),
        Mm(150.0),
        &font,
    );
    current_layer.use_text(
        format!("This certifies that {} has completed", cert.trainee_name),
        20.0,
        Mm(60.0),
        Mm(120.0),
        &font,
    );
    current_layer.use_text(
        format!("the course: {}", cert.course_title),
        20.0,
        Mm(80.0),
        Mm(100.0),
        &font,
    );

    let mut buffer = Vec::new();
    {
        let mut writer = BufWriter::new(&mut buffer);
        doct.save(&mut writer)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let filename = format!("certificate_{}.pdf", cert.credential_id);
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/pdf".parse().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", filename)
            .parse()
            .unwrap(),
    );

    Ok((headers, buffer).into_response())
}

// ============================================================================
// Routes
// ============================================================================

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/attendance", get(generate_attendance_report))
        .route("/certificate/{id}", get(generate_certificate_pdf))
}
