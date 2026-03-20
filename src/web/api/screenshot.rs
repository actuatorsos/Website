//! Screenshot API — لقطات شاشة حية للمواقع
//!
//! GET /api/screenshot?url=https://example.com
//! يُرجع صورة PNG. يخزّن النتيجة مؤقتاً لمدة 24 ساعة.

use axum::{
    Router,
    extract::Query,
    response::IntoResponse,
    routing::get,
};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct ScreenshotParams {
    url: String,
}

/// يحوّل الرابط لاسم ملف آمن للتخزين المؤقت
fn url_to_cache_key(url: &str) -> String {
    let clean: String = url
        .replace("https://", "")
        .replace("http://", "")
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '.' { c } else { '_' })
        .collect();
    format!("{}.png", clean)
}

fn cache_dir() -> PathBuf {
    let dir = PathBuf::from("static/images/cache");
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }
    dir
}

/// يتحقق إذا الكاش صالح (أقل من 24 ساعة)
fn cached_file(key: &str) -> Option<Vec<u8>> {
    let path = cache_dir().join(key);
    if !path.exists() {
        return None;
    }
    if let Ok(meta) = std::fs::metadata(&path) {
        if let Ok(modified) = meta.modified() {
            let age = std::time::SystemTime::now()
                .duration_since(modified)
                .unwrap_or_default();
            // 24 ساعة
            if age.as_secs() < 86400 {
                return std::fs::read(&path).ok();
            }
        }
    }
    None
}

/// يأخذ لقطة شاشة باستخدام headless Chrome
fn take_screenshot(url: &str) -> Result<Vec<u8>, String> {
    use headless_chrome::{Browser, LaunchOptions};
    use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;

    let options = LaunchOptions {
        headless: true,
        window_size: Some((1280, 800)),
        ..LaunchOptions::default()
    };

    let browser = Browser::new(options).map_err(|e| format!("Browser launch failed: {}", e))?;
    let tab = browser.new_tab().map_err(|e| format!("New tab failed: {}", e))?;

    tab.navigate_to(url)
        .map_err(|e| format!("Navigate failed: {}", e))?;

    // انتظر تحميل الصفحة
    tab.wait_until_navigated()
        .map_err(|e| format!("Wait failed: {}", e))?;

    // انتظر إضافي للمحتوى الديناميكي
    std::thread::sleep(std::time::Duration::from_secs(2));

    let png_data = tab
        .capture_screenshot(CaptureScreenshotFormatOption::Png, None, None, true)
        .map_err(|e| format!("Screenshot failed: {}", e))?;

    Ok(png_data)
}

async fn screenshot_handler(
    Query(params): Query<ScreenshotParams>,
) -> impl IntoResponse {
    // تحقق أن الرابط يبدأ بـ http
    if !params.url.starts_with("http://") && !params.url.starts_with("https://") {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            [("content-type", "application/json")],
            b"{\"error\":\"URL must start with http:// or https://\"}".to_vec(),
        );
    }

    let cache_key = url_to_cache_key(&params.url);

    // تحقق من الكاش
    if let Some(data) = cached_file(&cache_key) {
        return (
            axum::http::StatusCode::OK,
            [("content-type", "image/png")],
            data,
        );
    }

    // أخذ اللقطة في thread منفصل (لأن headless_chrome متزامن)
    let url = params.url.clone();
    let key = cache_key.clone();

    let result = tokio::task::spawn_blocking(move || {
        let data = take_screenshot(&url)?;
        // خزّن في الكاش
        let path = cache_dir().join(&key);
        let _ = std::fs::write(&path, &data);
        Ok::<Vec<u8>, String>(data)
    })
    .await;

    match result {
        Ok(Ok(data)) => (
            axum::http::StatusCode::OK,
            [("content-type", "image/png")],
            data,
        ),
        Ok(Err(e)) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            [("content-type", "application/json")],
            format!("{{\"error\":\"{}\"}}", e).into_bytes(),
        ),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            [("content-type", "application/json")],
            format!("{{\"error\":\"Task failed: {}\"}}", e).into_bytes(),
        ),
    }
}

pub fn routes() -> Router<crate::db::AppState> {
    Router::new().route("/", get(screenshot_handler))
}
