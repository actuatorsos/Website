use super::models::{DocumentLink, DocumentProperty, DocumentResponse, UpdateDocumentRequest};
use crate::db::{AppState, DbError};
use serde_json::Value;

pub async fn get_document_profile(
    state: &AppState,
    entity_type: &str,
    id: &str,
) -> Result<DocumentResponse, DbError> {
    let raw_id = id.to_string();
    let record_id = format!("{}:{}", entity_type, raw_id);

    // We will query the base record from DB
    let record: Option<Value> = state.db.select((entity_type, raw_id.clone())).await?;
    let record = record.ok_or(DbError::NotFound)?;

    let icon = record
        .get("icon")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let cover_image = record
        .get("cover_image")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let content_md = record
        .get("content_md")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let mut properties = Vec::new();
    let mut links = Vec::new();
    let files = Vec::new(); // Files will be implemented later, or read from "files" edge

    let title = match entity_type {
        "customer" | "client" => {
            if let Some(phone) = record.get("phone").and_then(|v| v.as_str()) {
                properties.push(DocumentProperty {
                    key: "phone".into(),
                    label: "رقم الهاتف".into(),
                    value: phone.into(),
                });
            }
            if let Some(company) = record.get("company").and_then(|v| v.as_str()) {
                properties.push(DocumentProperty {
                    key: "company".into(),
                    label: "الشركة".into(),
                    value: company.into(),
                });
            }

            // Fetch connected machines
            let mut machines: surrealdb::Response = state.db.query("SELECT id, model, manufacturer FROM machine WHERE customer_id = $id OR client = type::thing('client', $id)")
                .bind(("id", raw_id.clone()))
                .await?;
            let machines: Vec<Value> = machines.take(0).unwrap_or_default();
            for m in machines {
                links.push(DocumentLink {
                    id: m
                        .get("id")
                        .unwrap()
                        .to_string()
                        .replace("\"", "")
                        .replace("machine:", ""),
                    title: format!(
                        "{} {}",
                        m.get("manufacturer")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown"),
                        m.get("model").and_then(|v| v.as_str()).unwrap_or("Unknown")
                    ),
                    relation: "آلة مرتبطة".to_string(),
                    icon: Some("⚙️".to_string()),
                });
            }

            // Fetch connected projects
            let mut projects: surrealdb::Response = state.db.query("SELECT id, title, name FROM project WHERE customer_id = $id OR client = type::thing('client', $id)")
                .bind(("id", raw_id.clone()))
                .await?;
            let projects: Vec<Value> = projects.take(0).unwrap_or_default();
            for p in projects {
                let p_title = p
                    .get("title")
                    .or(p.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string();
                links.push(DocumentLink {
                    id: p
                        .get("id")
                        .unwrap()
                        .to_string()
                        .replace("\"", "")
                        .replace("project:", ""),
                    title: p_title,
                    relation: "مشروع".to_string(),
                    icon: Some("📁".to_string()),
                });
            }

            // Fetch Recent Interactions / Visits
            let mut interactions: surrealdb::Response = state.db.query("SELECT id, subject, channel, created_at FROM interaction WHERE client = type::thing('client', $id) ORDER BY created_at DESC LIMIT 5")
                .bind(("id", raw_id.clone()))
                .await?;
            let interactions: Vec<Value> = interactions.take(0).unwrap_or_default();
            for i in interactions {
                links.push(DocumentLink {
                    id: i
                        .get("id")
                        .unwrap()
                        .to_string()
                        .replace("\"", "")
                        .replace("interaction:", ""),
                    title: i
                        .get("subject")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Interaction")
                        .to_string(),
                    relation: "تفاعل/زيارة".to_string(),
                    icon: Some("💬".to_string()),
                });
            }

            // Fetch Repair Operations
            let mut repairs: surrealdb::Response = state.db.query("SELECT id, description, status FROM repair_operation WHERE customer_id = $id OR machine_id.customer_id = $id ORDER BY created_at DESC LIMIT 5")
                .bind(("id", raw_id.clone()))
                .await?;
            let repairs: Vec<Value> = repairs.take(0).unwrap_or_default();
            for r in repairs {
                links.push(DocumentLink {
                    id: r
                        .get("id")
                        .unwrap()
                        .to_string()
                        .replace("\"", "")
                        .replace("repair_operation:", ""),
                    title: r
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Repair")
                        .to_string(),
                    relation: "عملية إصلاح".to_string(),
                    icon: Some("🛠️".to_string()),
                });
            }

            record
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown Client")
                .to_string()
        }
        "machine" => {
            let model = record
                .get("model")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown Model");
            let mfg = record
                .get("manufacturer")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown Manufacturer");

            if let Some(sn) = record.get("serial_number").and_then(|v| v.as_str()) {
                properties.push(DocumentProperty {
                    key: "serial_number".into(),
                    label: "الرقم التسلسلي".into(),
                    value: sn.into(),
                });
            }
            if let Some(status) = record.get("status").and_then(|v| v.as_str()) {
                properties.push(DocumentProperty {
                    key: "status".into(),
                    label: "الحالة".into(),
                    value: status.into(),
                });
            }

            // Link back to client if possible
            if let Some(client_id) = record.get("customer_id").and_then(|v| v.as_str()) {
                if !client_id.is_empty() {
                    let client_val: Option<Value> =
                        state.db.select(("client", client_id.clone())).await?;
                    if let Some(c) = client_val {
                        links.push(DocumentLink {
                            id: client_id.to_string(),
                            title: c
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown Client")
                                .to_string(),
                            relation: "المالك".to_string(),
                            icon: Some("👤".to_string()),
                        });
                    }
                }
            }

            // Fetch Repair Operations for this machine
            let mut repairs: surrealdb::Response = state.db.query("SELECT id, description FROM repair_operation WHERE machine_id = type::thing('machine', $id) ORDER BY created_at DESC")
                .bind(("id", raw_id.clone()))
                .await?;
            let repairs: Vec<Value> = repairs.take(0).unwrap_or_default();
            for r in repairs {
                links.push(DocumentLink {
                    id: r
                        .get("id")
                        .unwrap()
                        .to_string()
                        .replace("\"", "")
                        .replace("repair_operation:", ""),
                    title: r
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Repair")
                        .to_string(),
                    relation: "سجل إصلاح".to_string(),
                    icon: Some("🛠️".to_string()),
                });
            }

            format!("{} {}", mfg, model)
        }
        "project" => {
            if let Some(status) = record.get("status").and_then(|v| v.as_str()) {
                properties.push(DocumentProperty {
                    key: "status".into(),
                    label: "الحالة".into(),
                    value: status.into(),
                });
            }

            if let Some(client_id) = record
                .get("customer_id")
                .or(record.get("client_id"))
                .and_then(|v| v.as_str())
            {
                if !client_id.is_empty() {
                    let client_val: Option<Value> =
                        state.db.select(("client", client_id.clone())).await?;
                    if let Some(c) = client_val {
                        links.push(DocumentLink {
                            id: client_id.to_string(),
                            title: c
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown Client")
                                .to_string(),
                            relation: "العميل".to_string(),
                            icon: Some("👤".to_string()),
                        });
                    }
                }
            }

            // Fetch Project Members
            let mut members: surrealdb::Response = state.db.query("SELECT member.name as name, role, member as member_id FROM project_member WHERE project = type::thing('project', $id)")
                .bind(("id", raw_id.clone()))
                .await?;
            let members: Vec<Value> = members.take(0).unwrap_or_default();
            for m in members {
                links.push(DocumentLink {
                    id: m
                        .get("member_id")
                        .unwrap()
                        .to_string()
                        .replace("\"", "")
                        .replace("account:", ""),
                    title: m
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Member")
                        .to_string(),
                    relation: format!(
                        "عضو ({})",
                        m.get("role").and_then(|v| v.as_str()).unwrap_or("—")
                    ),
                    icon: Some("👥".to_string()),
                });
            }

            record
                .get("title")
                .or(record.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown Project")
                .to_string()
        }
        "user" | "employee" | "account" => record
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(entity_type)
            .to_string(),
        _ => record
            .get("name")
            .or(record.get("title"))
            .and_then(|v| v.as_str())
            .unwrap_or(entity_type)
            .to_string(),
    };

    Ok(DocumentResponse {
        id: record_id,
        entity_type: entity_type.to_string(),
        title,
        icon,
        cover_image,
        content_md,
        properties,
        links,
        files,
    })
}

pub async fn update_document(
    state: &AppState,
    entity_type: &str,
    id: &str,
    req: UpdateDocumentRequest,
) -> Result<(), DbError> {
    let raw_id = id.to_string();

    let mut merge_data = serde_json::Map::new();
    if let Some(title) = req.title {
        if entity_type == "client"
            || entity_type == "machine"
            || entity_type == "employee"
            || entity_type == "account"
        {
            merge_data.insert("name".to_string(), Value::String(title));
        } else {
            merge_data.insert("title".to_string(), Value::String(title));
        }
    }
    if let Some(icon) = req.icon {
        merge_data.insert("icon".to_string(), Value::String(icon));
    }
    if let Some(cover_image) = req.cover_image {
        merge_data.insert("cover_image".to_string(), Value::String(cover_image));
    }
    if let Some(content_md) = req.content_md {
        merge_data.insert("content_md".to_string(), Value::String(content_md));
    }

    if !merge_data.is_empty() {
        let _: Option<Value> = state
            .db
            .update((entity_type, raw_id))
            .merge(Value::Object(merge_data))
            .await?;
    }

    Ok(())
}
