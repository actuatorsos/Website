//! Public Routes
//!
//! صفحات الويب العامة

use askama::Template;
use axum::{
    Router,
    extract::{Query, State},
    response::{Html, IntoResponse, Redirect},
    routing::get,
};
use std::collections::HashMap;
use tower_cookies::{Cookie, Cookies};

use crate::db::AppState;
use crate::i18n::Language;

// ============================================================================
// Templates
// ============================================================================

/// Template for the home page
#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    /// Current language code
    pub lang: String,
    /// Text direction (ltr/rtl)
    pub dir: String,
    /// Translations
    pub t: HashMap<String, String>,
}


// ============================================================================
// Helpers
// ============================================================================

/// Query parameter for language selection
#[derive(serde::Deserialize)]
pub struct LangParam {
    /// Language code (e.g., "en", "ar")
    lang: Option<String>,
}

fn resolve_language(cookies: &Cookies, query: Option<String>) -> Language {
    // 1. Check Query Param
    if let Some(l) = query {
        let lang = Language::from_str(&l);
        // Update cookie
        cookies.add(Cookie::new("lang", lang.as_str().to_string()));
        return lang;
    }

    // 2. Check Cookie
    if let Some(cookie) = cookies.get("lang") {
        return Language::from_str(cookie.value());
    }

    // 3. Default
    Language::Arabic
}

// ============================================================================
// Handlers
// ============================================================================

async fn index(
    State(state): State<AppState>,
    cookies: Cookies,
    Query(params): Query<LangParam>,
) -> impl IntoResponse {
    let lang = resolve_language(&cookies, params.lang);
    let t = state.i18n.get_dictionary(lang.as_str());

    let template = IndexTemplate {
        lang: lang.as_str().to_string(),
        dir: lang.dir().to_string(),
        t: {
            println!(
                "Request /: Lang={}, Dictionary Keys={}",
                lang.as_str(),
                t.len()
            );
            t
        },
    };
    Html(
        template
            .render()
            .unwrap_or_else(|e| format!("Error: {}", e)),
    )
}

async fn about(
    Query(params): Query<LangParam>,
) -> impl IntoResponse {
    let lang_param = params.lang.unwrap_or_default();
    if lang_param.is_empty() {
        Redirect::permanent("/#about")
    } else {
        Redirect::permanent(&format!("/?lang={}#about", lang_param))
    }
}

async fn services(
    Query(params): Query<LangParam>,
) -> impl IntoResponse {
    let lang_param = params.lang.unwrap_or_default();
    if lang_param.is_empty() {
        Redirect::permanent("/#services")
    } else {
        Redirect::permanent(&format!("/?lang={}#services", lang_param))
    }
}

// ============================================================================
// Routes
// ============================================================================

/// Configures routes for public pages
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/about", get(about))
        .route("/services", get(services))
}
