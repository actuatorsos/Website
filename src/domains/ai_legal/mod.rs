//! AI Legal Domain — المستشار القانوني السوري الذكي
//!
//! RAG + Claude API for Syrian legal advisory chatbot

pub mod handlers;
pub mod models;
pub mod repository;
pub mod service;

pub use handlers::legal_routes;
