//! Finance Domain Models
//!
//! نماذج بيانات الشؤون المالية (فواتير، شهادات)

use f64;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use validator::Validate;

// ============================================================================
// Invoice Models
// ============================================================================

/// Invoice status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InvoiceStatus {
    /// Invoice is a draft, not yet sent
    Draft,
    /// Invoice has been sent to the client
    Sent,
    /// Invoice is partially paid
    PartiallyPaid,
    /// Invoice is fully paid
    Paid,
    /// Invoice has been cancelled
    Cancelled,
    /// Invoice is overdue
    Overdue,
}

impl std::fmt::Display for InvoiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvoiceStatus::Draft => write!(f, "مسودة"),
            InvoiceStatus::Sent => write!(f, "مُرسلة"),
            InvoiceStatus::PartiallyPaid => write!(f, "مدفوعة جزئياً"),
            InvoiceStatus::Paid => write!(f, "مدفوعة"),
            InvoiceStatus::Cancelled => write!(f, "ملغاة"),
            InvoiceStatus::Overdue => write!(f, "متأخرة"),
        }
    }
}

/// Invoice model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    /// Unique identifier
    pub id: Option<Thing>,
    /// Invoice number (auto-generated, e.g. INV-2026-001)
    pub invoice_number: String,
    /// Client ID (reference to clients table)
    pub client_id: String,
    /// Client name (cached for display)
    pub client_name: Option<String>,
    /// Invoice date (YYYY-MM-DD)
    pub invoice_date: String,
    /// Due date (YYYY-MM-DD)
    pub due_date: Option<String>,
    /// Current status
    pub status: InvoiceStatus,
    /// Line items
    pub items: Vec<InvoiceItem>,
    /// Subtotal before tax
    pub subtotal: f64,
    /// Tax rate as percentage (e.g. 15.0 for 15%)
    pub tax_rate: f64,
    /// Tax amount
    pub tax_amount: f64,
    /// Total amount including tax
    pub total: f64,
    /// Amount already paid
    pub paid_amount: f64,
    /// Remaining balance
    pub balance: f64,
    /// Optional notes
    pub notes: Option<String>,
    /// Creation timestamp
    pub created_at: Option<String>,
}

/// Invoice line item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceItem {
    /// Item description
    pub description: String,
    /// Quantity
    pub quantity: f64,
    /// Unit price
    pub unit_price: f64,
    /// Line total (quantity * unit_price)
    pub total: f64,
}

impl Invoice {
    /// Get CSS class for status badge
    pub fn status_class(&self) -> &str {
        match self.status {
            InvoiceStatus::Draft => "bg-gray-100 text-gray-800",
            InvoiceStatus::Sent => "bg-blue-100 text-blue-800",
            InvoiceStatus::PartiallyPaid => "bg-yellow-100 text-yellow-800",
            InvoiceStatus::Paid => "bg-green-100 text-green-800",
            InvoiceStatus::Cancelled => "bg-red-100 text-red-800",
            InvoiceStatus::Overdue => "bg-orange-100 text-orange-800",
        }
    }

    /// Get the ID as a string for template use
    pub fn id_string(&self) -> String {
        self.id
            .as_ref()
            .map(|thing| thing.id.to_string())
            .unwrap_or_default()
    }

    /// Format total as currency string
    pub fn total_display(&self) -> String {
        format!("{:.2}", self.total)
    }

    /// Format balance as currency string
    pub fn balance_display(&self) -> String {
        format!("{:.2}", self.balance)
    }

    /// Recalculate totals from items
    pub fn recalculate(&mut self) {
        self.subtotal = self.items.iter().map(|i| i.total).sum();
        self.tax_amount = self.subtotal * (self.tax_rate / f64::from(100));
        self.total = self.subtotal + self.tax_amount;
        self.balance = self.total - self.paid_amount;
    }

    /// Check if invoice is a draft (for template use)
    pub fn is_draft(&self) -> bool {
        self.status == InvoiceStatus::Draft
    }

    /// Check if invoice can accept payments (for template use)
    pub fn can_pay(&self) -> bool {
        matches!(
            self.status,
            InvoiceStatus::Sent | InvoiceStatus::PartiallyPaid | InvoiceStatus::Overdue
        )
    }
}

/// Request to create a new invoice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInvoiceRequest {
    /// Client ID
    pub client_id: String,
    /// Invoice date
    pub invoice_date: String,
    /// Due date
    pub due_date: Option<String>,
    /// Line items
    pub items: Vec<InvoiceItem>,
    /// Tax rate percentage
    pub tax_rate: f64,
    /// Notes
    pub notes: Option<String>,
}

/// Request to update invoice status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInvoiceStatusRequest {
    /// New status
    pub status: InvoiceStatus,
}

/// Request to record a payment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordPaymentRequest {
    /// Payment amount
    pub amount: f64,
    /// Payment date
    pub payment_date: String,
    /// Payment method (cash, bank_transfer, etc.)
    pub payment_method: Option<String>,
    /// Reference number
    pub reference: Option<String>,
}

// ============================================================================
// Certificate Models
// ============================================================================

/// Certificate status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CertificateStatus {
    Issued,
    Revoked,
}

impl std::fmt::Display for CertificateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CertificateStatus::Issued => write!(f, "issued"),
            CertificateStatus::Revoked => write!(f, "revoked"),
        }
    }
}

/// Certificate record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    pub id: Option<Thing>,
    pub credential_id: String,
    pub trainee_name: String,
    pub course_title: String,
    pub total_hours: f64,
    pub start_date: String,
    pub end_date: String,
    pub instructor: String,
    pub issued_date: String,
    pub status: CertificateStatus,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCertificateRequest {
    #[validate(length(min = 2, max = 100))]
    pub trainee_name: String,
    #[validate(length(min = 2, max = 100))]
    pub course_title: String,
    #[validate(range(min = 0.0))]
    pub total_hours: f64,
    #[validate(length(min = 8, max = 50))]
    pub start_date: String,
    #[validate(length(min = 8, max = 50))]
    pub end_date: String,
    #[validate(length(min = 2, max = 100))]
    pub instructor: String,
}

impl Certificate {
    /// Extract record ID as string
    pub fn id_string(&self) -> String {
        self.id
            .as_ref()
            .map(|thing| thing.id.to_string())
            .unwrap_or_default()
    }

    /// Status CSS class
    pub fn status_class(&self) -> &str {
        match self.status {
            CertificateStatus::Issued => "bg-success/10 text-success",
            CertificateStatus::Revoked => "bg-error/10 text-error",
        }
    }

    /// Status display text
    pub fn status_text(&self) -> &str {
        match self.status {
            CertificateStatus::Issued => "صادرة",
            CertificateStatus::Revoked => "ملغاة",
        }
    }
}
