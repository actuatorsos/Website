use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentProperty {
    pub key: String,
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentLink {
    pub id: String,       // e.g. "machine:123"
    pub title: String,    // e.g. "Wood-6AV-1"
    pub relation: String, // e.g. "owned_machines" or "projects"
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentFile {
    pub id: String,
    pub name: String,
    pub url: String,
    pub file_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentResponse {
    pub id: String,
    pub entity_type: String, // "client", "machine", "project", etc.
    pub title: String,
    pub icon: Option<String>,
    pub cover_image: Option<String>,
    pub content_md: Option<String>,
    pub properties: Vec<DocumentProperty>,
    pub links: Vec<DocumentLink>,
    pub files: Vec<DocumentFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDocumentRequest {
    pub title: Option<String>,
    pub icon: Option<String>,
    pub cover_image: Option<String>,
    pub content_md: Option<String>,
}
