//! Seed Data — Default data initialization
//!
//! Runs once on server startup if data does not already exist

use surrealdb::Surreal;
use surrealdb::engine::remote::ws::Client;

/// Seed all default data
pub async fn seed_all(db: &Surreal<Client>) {
    seed_company_settings(db).await;
    seed_departments(db).await;
    seed_positions(db).await;
    seed_account_chart(db).await;
    seed_product_categories(db).await;
    fix_null_defaults(db).await;
    tracing::info!("✅ Seed data completed");
}

/// Fix records with NULL/NONE fields
/// Includes: created_at (datetime) + base_salary/housing/transport (decimal) + dependents (int)
async fn fix_null_defaults(db: &Surreal<Client>) {
    // ── 1. Fix created_at for all tables ──
    let tables = [
        "trainee",
        "account",
        "employee",
        "asset",
        "machine",
        "project",
        "repair_operation",
        "invoice",
        "client",
        "certificate",
        "attendance",
    ];
    for table in tables {
        let query = format!("UPDATE {table} SET created_at = time::now() WHERE created_at IS NONE");
        if let Err(e) = db.query(&query).await {
            tracing::warn!("⚠️ Fix timestamps for {table}: {e}");
        }
    }

    // ── 2. Fix employee defaults ──
    let emp_query = r#"
        UPDATE employee SET 
            base_salary = 0, 
            dependents = 0, 
            housing_allowance = 0, 
            transport_allowance = 0, 
            employment_type = 'full_time',
            status = 'active'
        WHERE base_salary IS NONE OR employment_type IS NONE
    "#;

    if let Err(e) = db.query(emp_query).await {
        tracing::warn!("⚠️ Fix employee defaults: {e}");
    }

    tracing::info!("✅ Null defaults fix completed");
}

// ──────────────────────────────────────────────────────────────────
// Company Settings
// ──────────────────────────────────────────────────────────────────
async fn seed_company_settings(db: &Surreal<Client>) {
    let exists: Vec<serde_json::Value> = db
        .query("SELECT id FROM company_settings LIMIT 1")
        .await
        .and_then(|mut r| r.take(0))
        .unwrap_or_default();

    if !exists.is_empty() {
        return;
    }

    let _ = db
        .query(
            r#"
        CREATE company_settings SET
            company_name   = 'Dr.Machine',
            company_name_en = 'Dr.Machine',
            vat_number     = '300000000000003',
            cr_number      = '1010000000',
            address        = 'Riyadh, Saudi Arabia',
            phone          = '+966500000000',
            email          = 'info@drmachine.sa',
            currency       = 'SAR',
            fiscal_year_start = '01-01',
            vat_rate       = 15.0,
            invoice_prefix = 'INV',
            po_prefix      = 'PO',
            is_zatca_enabled = false
    "#,
        )
        .await;

    tracing::info!("✅ Company settings seeded");
}

// ──────────────────────────────────────────────────────────────────
// Departments — Default departments
// ──────────────────────────────────────────────────────────────────
async fn seed_departments(db: &Surreal<Client>) {
    let exists: Vec<serde_json::Value> = db
        .query("SELECT id FROM department LIMIT 1")
        .await
        .and_then(|mut r| r.take(0))
        .unwrap_or_default();

    if !exists.is_empty() {
        return;
    }

    let departments = vec![
        ("management", "General Management"),
        ("hr", "Human Resources"),
        ("finance", "Finance and Accounting"),
        ("sales", "Sales and Marketing"),
        ("technical", "Technical Support"),
        ("warehouse", "Warehouse"),
        ("production", "Production and Manufacturing"),
    ];

    for (code, name) in departments {
        let _ = db.query(format!(
            "CREATE department SET code = '{}', name = '{}', is_active = true",
            code, name
        )).await;
    }

    tracing::info!("✅ Departments seeded");
}

// ──────────────────────────────────────────────────────────────────
// Positions — Default positions
// ──────────────────────────────────────────────────────────────────
async fn seed_positions(db: &Surreal<Client>) {
    let exists: Vec<serde_json::Value> = db
        .query("SELECT id FROM position LIMIT 1")
        .await
        .and_then(|mut r| r.take(0))
        .unwrap_or_default();

    if !exists.is_empty() {
        return;
    }

    let positions = vec![
        ("ceo", "Chief Executive Officer", "management", 1),
        ("department_manager", "Department Manager", "management", 2),
        ("hr_specialist", "HR Specialist", "hr", 3),
        ("accountant", "Accountant", "finance", 3),
        ("sales_rep", "Sales Representative", "sales", 4),
        ("technician", "Technician", "technical", 4),
        ("senior_tech", "Senior Technician", "technical", 3),
        ("warehouse_keeper", "Warehouse Keeper", "warehouse", 4),
        ("production_worker", "Production Worker", "production", 5),
        ("driver", "Driver", "management", 5),
    ];

    for (code, title, dept_code, grade) in positions {
        let _ = db
            .query(format!(
                "CREATE position SET code = '{}', title = '{}', grade = '{}', \
             department = (SELECT id FROM department WHERE code = '{}' LIMIT 1)[0], \
             is_active = true",
                code, title, grade, dept_code
            ))
            .await;
    }

    tracing::info!("✅ Positions seeded");
}

// ──────────────────────────────────────────────────────────────────
// Account Chart — Chart of Accounts
// ──────────────────────────────────────────────────────────────────
async fn seed_account_chart(db: &Surreal<Client>) {
    let exists: Vec<serde_json::Value> = db
        .query("SELECT id FROM account_chart LIMIT 1")
        .await
        .and_then(|mut r| r.take(0))
        .unwrap_or_default();

    if !exists.is_empty() {
        return;
    }

    // (code, name, type, parent_code, is_header)
    let accounts: Vec<(&str, &str, &str, Option<&str>, bool)> = vec![
        // ── Assets
        ("1000", "Assets", "asset", None, true),
        ("1100", "Current Assets", "asset", Some("1000"), true),
        ("1110", "Cash", "asset", Some("1100"), false),
        ("1111", "Bank - Main Account", "asset", Some("1100"), false),
        ("1120", "Accounts Receivable", "asset", Some("1100"), false),
        ("1130", "Inventory", "asset", Some("1100"), false),
        ("1140", "VAT - Input Tax", "asset", Some("1100"), false),
        ("1200", "Non-Current Assets", "asset", Some("1000"), true),
        ("1210", "Equipment and Machinery", "asset", Some("1200"), false),
        ("1211", "Accumulated Depreciation - Equipment", "asset", Some("1200"), false),
        ("1220", "Furniture and Fixtures", "asset", Some("1200"), false),
        ("1230", "Computers and Devices", "asset", Some("1200"), false),
        // ── Liabilities
        ("2000", "Liabilities", "liability", None, true),
        ("2100", "Current Liabilities", "liability", Some("2000"), true),
        ("2110", "Accounts Payable", "liability", Some("2100"), false),
        ("2120", "VAT - Output Tax", "liability", Some("2100"), false),
        ("2130", "Salaries Payable", "liability", Some("2100"), false),
        ("2140", "Accrued Expenses", "liability", Some("2100"), false),
        ("2200", "Non-Current Liabilities", "liability", Some("2000"), true),
        ("2210", "Long-Term Loans", "liability", Some("2200"), false),
        // ── Equity
        ("3000", "Equity", "equity", None, true),
        ("3100", "Capital", "equity", Some("3000"), false),
        ("3200", "Retained Earnings", "equity", Some("3000"), false),
        ("3300", "Period Profit or Loss", "equity", Some("3000"), false),
        // ── Revenue
        ("4000", "Revenue", "revenue", None, true),
        ("4100", "Sales Revenue", "revenue", Some("4000"), false),
        ("4110", "Service Revenue", "revenue", Some("4000"), false),
        ("4120", "Maintenance Revenue", "revenue", Some("4000"), false),
        ("4130", "Spare Parts Revenue", "revenue", Some("4000"), false),
        ("4900", "Other Revenue", "revenue", Some("4000"), false),
        // ── Expenses
        ("5000", "Expenses", "expense", None, true),
        ("5100", "Cost of Sales", "expense", Some("5000"), true),
        ("5110", "Cost of Goods Sold", "expense", Some("5100"), false),
        ("5120", "Cost of Services", "expense", Some("5100"), false),
        ("5200", "Operating Expenses", "expense", Some("5000"), true),
        ("5210", "Salaries and Wages", "expense", Some("5200"), false),
        ("5220", "Rent", "expense", Some("5200"), false),
        ("5230", "Utilities", "expense", Some("5200"), false),
        ("5240", "Telecommunications", "expense", Some("5200"), false),
        ("5250", "Maintenance and Repair", "expense", Some("5200"), false),
        ("5260", "Depreciation", "expense", Some("5200"), false),
        ("5270", "Insurance", "expense", Some("5200"), false),
        ("5280", "Office Supplies", "expense", Some("5200"), false),
        ("5300", "Marketing Expenses", "expense", Some("5000"), false),
        ("5400", "Administrative Expenses", "expense", Some("5000"), false),
        ("5900", "Other Expenses", "expense", Some("5000"), false),
    ];

    let total_accounts = accounts.len();
    for (code, name, acc_type, parent_code, is_header) in accounts {
        let parent_q = match parent_code {
            Some(p) => format!(
                "(SELECT id FROM account_chart WHERE code = '{}' LIMIT 1)[0]",
                p
            ),
            None => "NONE".to_string(),
        };
        let _ = db
            .query(format!(
                "CREATE account_chart SET code = '{}', name = '{}', type = '{}', \
             parent = {}, is_header = {}, is_active = true",
                code, name, acc_type, parent_q, is_header
            ))
            .await;
    }

    tracing::info!("✅ Account chart seeded ({} accounts)", total_accounts);
}

// ──────────────────────────────────────────────────────────────────
// Product Categories
// ──────────────────────────────────────────────────────────────────
async fn seed_product_categories(db: &Surreal<Client>) {
    let exists: Vec<serde_json::Value> = db
        .query("SELECT id FROM product_category LIMIT 1")
        .await
        .and_then(|mut r| r.take(0))
        .unwrap_or_default();

    if !exists.is_empty() {
        return;
    }

    let categories = vec![
        ("machines", "Machines and Equipment"),
        ("electronics", "Electronics"),
        ("spare_parts", "Spare Parts"),
        ("consumables", "Consumables"),
        ("tools", "Tools"),
        ("accessories", "Accessories"),
    ];

    for (code, name) in categories {
        let _ = db.query(format!(
            "CREATE product_category SET code = '{}', name = '{}', is_active = true",
            code, name
        )).await;
    }

    tracing::info!("✅ Product categories seeded");
}
