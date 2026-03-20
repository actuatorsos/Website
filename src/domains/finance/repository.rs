//! Finance Domain Database Logic
//!
//! فصل العمليات المتعلقة بقواعد البيانات للمالية كالفواتير والشهادات

use super::models::{
    Certificate, CertificateStatus, CreateCertificateRequest, CreateInvoiceRequest, Invoice,
    InvoiceItem, InvoiceStatus,
};
use crate::db::{AppState, DbError};
use f64;

// ============================================================================
// Invoice Repository
// ============================================================================

/// Generate next invoice number (INV-YYYY-NNN)
async fn next_invoice_number(state: &AppState) -> Result<String, DbError> {
    let year = chrono::Utc::now().format("%Y");
    let invoices: Vec<Invoice> = state
        .db
        .query("SELECT * FROM invoice ORDER BY created_at DESC LIMIT 1")
        .await?
        .take::<Vec<Invoice>>(0)?;

    let next_num = if let Some(last) = invoices.first() {
        let parts: Vec<&str> = last.invoice_number.split('-').collect();
        if parts.len() == 3 {
            parts[2].parse::<u32>().unwrap_or(0) + 1
        } else {
            1
        }
    } else {
        1
    };

    Ok(format!("INV-{}-{:03}", year, next_num))
}

pub async fn create_invoice(
    state: &AppState,
    req: CreateInvoiceRequest,
) -> Result<Invoice, DbError> {
    let invoice_number = next_invoice_number(state).await?;
    let now = chrono::Utc::now().to_rfc3339();

    // Look up client name
    let client_name = crate::domains::customers::repository::get_client(state, &req.client_id)
        .await
        .map(|c| c.company_name)
        .ok();

    // Calculate totals using Decimal (precise)
    let subtotal: f64 = req.items.iter().map(|i| i.quantity * i.unit_price).sum();
    let tax_amount = subtotal * (req.tax_rate / 100.0);
    let total = subtotal + tax_amount;

    let items_with_totals: Vec<InvoiceItem> = req
        .items
        .into_iter()
        .map(|i| InvoiceItem {
            description: i.description,
            quantity: i.quantity,
            unit_price: i.unit_price,
            total: i.quantity * i.unit_price,
        })
        .collect();

    let invoice = Invoice {
        id: None,
        invoice_number,
        client_id: req.client_id,
        client_name,
        invoice_date: req.invoice_date,
        due_date: req.due_date,
        status: InvoiceStatus::Draft,
        items: items_with_totals,
        subtotal,
        tax_rate: req.tax_rate,
        tax_amount,
        total,
        paid_amount: 0.0,
        balance: total,
        notes: req.notes,
        created_at: Some(now),
    };

    let created: Option<Invoice> = state
        .db
        .create::<Option<Invoice>>("invoice")
        .content(invoice)
        .await?
        .into_iter()
        .next();
    let inv = created.ok_or(DbError::NotFound)?;
    crate::db::audit_log(
        &state.db,
        None,
        "create",
        "invoice",
        inv.id.as_ref().map(|t| t.id.to_raw()).as_deref(),
        None,
        None,
    )
    .await?;
    Ok(inv)
}

pub async fn get_all_invoices(state: &AppState) -> Result<Vec<Invoice>, DbError> {
    let invoices: Vec<Invoice> = state.db
        .query("SELECT * FROM invoice WHERE is_archived = false OR is_archived = NONE ORDER BY created_at DESC")
        .await?
        .take::<Vec<Invoice>>(0)?;
    Ok(invoices)
}

pub async fn get_invoice(state: &AppState, id: &str) -> Result<Invoice, DbError> {
    let invoice: Option<Invoice> = state.db.select(("invoice", id)).await?;
    invoice.ok_or(DbError::NotFound)
}

pub async fn get_client_invoices(
    state: &AppState,
    client_id: &str,
) -> Result<Vec<Invoice>, DbError> {
    let cid = client_id.to_string();
    let invoices: Vec<Invoice> = state.db
        .query("SELECT * FROM invoice WHERE client_id = $cid AND (is_archived = false OR is_archived = NONE) ORDER BY created_at DESC")
        .bind(("cid", cid))
        .await?
        .take::<Vec<Invoice>>(0)?;
    Ok(invoices)
}

pub async fn update_invoice_status(
    state: &AppState,
    id: &str,
    status: InvoiceStatus,
) -> Result<Invoice, DbError> {
    let invoice: Option<Invoice> = state
        .db
        .update(("invoice", id))
        .merge(serde_json::json!({ "status": status }))
        .await?;
    invoice.ok_or(DbError::NotFound)
}

pub async fn record_invoice_payment(
    state: &AppState,
    id: &str,
    amount: f64,
) -> Result<Invoice, DbError> {
    let invoice = get_invoice(state, id).await?;
    let new_paid = invoice.paid_amount + amount;
    let new_balance = invoice.total - new_paid;
    let new_status = if new_balance <= 0.0 {
        InvoiceStatus::Paid
    } else {
        InvoiceStatus::PartiallyPaid
    };

    let updated: Option<Invoice> = state
        .db
        .update(("invoice", id))
        .merge(serde_json::json!({
            "paid_amount": new_paid,
            "balance": new_balance,
            "status": new_status,
        }))
        .await?;
    updated.ok_or(DbError::NotFound)
}

/// Soft delete — أرشفة الفاتورة (فقط المسودات)
pub async fn delete_invoice(state: &AppState, id: &str) -> Result<(), DbError> {
    let invoice = get_invoice(state, id).await?;
    if invoice.status != InvoiceStatus::Draft {
        return Err(DbError::Conflict(
            "لا يمكن حذف فاتورة غير مسودة. يمكنك إلغاؤها بدلاً من ذلك.".to_string(),
        ));
    }
    crate::db::soft_delete(&state.db, "invoice", id).await?;
    crate::db::audit_log(&state.db, None, "delete", "invoice", Some(id), None, None).await?;
    Ok(())
}

// ============================================================================
// Certificate Repository
// ============================================================================

pub async fn create_certificate(
    state: &AppState,
    req: CreateCertificateRequest,
) -> Result<Certificate, DbError> {
    let credential_id = uuid::Uuid::new_v4()
        .to_string()
        .chars()
        .take(8)
        .collect::<String>()
        .to_uppercase();
    let issued_date = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let cert: Option<Certificate> = state
        .db
        .create::<Option<Certificate>>("certificate")
        .content(Certificate {
            id: None,
            credential_id,
            trainee_name: req.trainee_name,
            course_title: req.course_title,
            total_hours: req.total_hours,
            start_date: req.start_date,
            end_date: req.end_date,
            instructor: req.instructor,
            issued_date,
            status: CertificateStatus::Issued,
        })
        .await?
        .into_iter()
        .next();
    let created = cert.ok_or(DbError::NotFound)?;
    crate::db::audit_log(
        &state.db,
        None,
        "create",
        "certificate",
        created.id.as_ref().map(|t| t.id.to_raw()).as_deref(),
        None,
        None,
    )
    .await?;
    Ok(created)
}

pub async fn get_all_certificates(state: &AppState) -> Result<Vec<Certificate>, DbError> {
    let certs: Vec<Certificate> = state.db
        .query("SELECT * FROM certificate WHERE is_archived = false OR is_archived = NONE ORDER BY issued_date DESC")
        .await?
        .take(0)?;
    Ok(certs)
}

pub async fn get_certificate_by_credential_id(
    state: &AppState,
    credential_id: &str,
) -> Result<Certificate, DbError> {
    let mut result = state
        .db
        .query("SELECT * FROM certificate WHERE credential_id = $cid LIMIT 1")
        .bind(("cid", credential_id.to_string()))
        .await?;
    let certs: Vec<Certificate> = result.take(0)?;
    certs.into_iter().next().ok_or(DbError::NotFound)
}

/// Soft delete — أرشفة الشهادة
pub async fn delete_certificate(state: &AppState, id: &str) -> Result<(), DbError> {
    crate::db::soft_delete(&state.db, "certificate", id).await?;
    crate::db::audit_log(
        &state.db,
        None,
        "delete",
        "certificate",
        Some(id),
        None,
        None,
    )
    .await?;
    Ok(())
}

pub async fn revoke_certificate(state: &AppState, id: &str) -> Result<Certificate, DbError> {
    let cert: Option<Certificate> = state
        .db
        .update(("certificate", id))
        .merge(serde_json::json!({ "status": "revoked" }))
        .await?;
    let revoked = cert.ok_or(DbError::NotFound)?;
    crate::db::audit_log(
        &state.db,
        None,
        "update",
        "certificate",
        Some(id),
        None,
        Some(serde_json::json!({"status": "revoked"})),
    )
    .await?;
    Ok(revoked)
}
