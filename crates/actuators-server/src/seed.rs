//! Seed script: applies the platform schema migration and populates initial
//! data (plans, service catalog, super-admin account).
//!
//! Run with:
//! ```sh
//! cargo run --bin seed
//! ```

use actuators_auth::password::hash_password;
use actuators_core::config::AppConfig;
use actuators_db::platform::PlatformDb;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    tracing::info!("starting Actuators seed script");

    let config = AppConfig::from_env().expect("failed to load config");

    tracing::info!(url = %config.surrealdb_url, "connecting to SurrealDB");
    let platform_db = PlatformDb::connect(&config)
        .await
        .expect("failed to connect to platform DB");

    // ── 1. Apply platform schema migration ────────────────────────
    tracing::info!("applying platform schema migration");
    let migration = include_str!("../../../crates/actuators-db/src/migrations/platform_v001.surql");
    platform_db
        .db
        .query(migration)
        .await
        .expect("failed to apply platform schema migration");
    tracing::info!("platform schema migration applied");

    // ── 2. Seed subscription plans ────────────────────────────────
    tracing::info!("seeding subscription plans");

    let plans = vec![
        serde_json::json!({
            "name": "free",
            "name_ar": "مجاني",
            "price_eur": 0.0,
            "max_users": 3,
            "storage_mb": 512,
            "services": ["auth", "settings", "audit", "projects", "documents"],
            "is_active": true
        }),
        serde_json::json!({
            "name": "basic",
            "name_ar": "أساسي",
            "price_eur": 29.0,
            "max_users": 10,
            "storage_mb": 5120,
            "services": [
                "auth", "settings", "audit", "projects", "documents",
                "hr", "attendance", "payroll", "accounting", "crm", "store"
            ],
            "is_active": true
        }),
        serde_json::json!({
            "name": "professional",
            "name_ar": "احترافي",
            "price_eur": 79.0,
            "max_users": 50,
            "storage_mb": 25600,
            "services": [
                "auth", "settings", "audit", "projects", "documents",
                "hr", "attendance", "payroll", "accounting", "crm", "store",
                "inventory", "manufacturing", "assets", "compliance", "training",
                "field_service", "ai_agents", "grants", "investor_dashboard"
            ],
            "is_active": true
        }),
        serde_json::json!({
            "name": "enterprise",
            "name_ar": "مؤسسي",
            "price_eur": 179.0,
            "max_users": 500,
            "storage_mb": 102400,
            "services": [
                "auth", "settings", "audit", "projects", "documents",
                "hr", "attendance", "payroll", "accounting", "crm", "store",
                "inventory", "manufacturing", "assets", "compliance", "training",
                "field_service", "ai_agents", "grants", "investor_dashboard",
                "iot", "robot_fleet", "api_marketplace", "enterprise_contracts",
                "video_training", "energy_monitoring", "hardware_bom",
                "invoicing", "customer_portal", "product_catalog", "customer_management"
            ],
            "is_active": true
        }),
    ];

    for plan in &plans {
        let name = plan["name"].as_str().unwrap();
        let existing = platform_db.get_plan_by_name(name).await;
        if existing.is_ok() && existing.unwrap().is_some() {
            tracing::info!(name, "plan already exists, skipping");
            continue;
        }
        platform_db
            .db
            .query(
                "CREATE plan SET
                    name       = $name,
                    name_ar    = $name_ar,
                    price_eur  = $price_eur,
                    max_users  = $max_users,
                    storage_mb = $storage_mb,
                    services   = $services,
                    is_active  = $is_active,
                    created_at = time::now()
                ;",
            )
            .bind(("name", plan["name"].as_str().unwrap().to_owned()))
            .bind(("name_ar", plan["name_ar"].as_str().unwrap().to_owned()))
            .bind(("price_eur", plan["price_eur"].as_f64().unwrap()))
            .bind(("max_users", plan["max_users"].as_i64().unwrap() as i32))
            .bind(("storage_mb", plan["storage_mb"].as_i64().unwrap() as i32))
            .bind(("services", plan["services"].as_array().unwrap().clone()))
            .bind(("is_active", true))
            .await
            .unwrap_or_else(|e| panic!("failed to create plan '{name}': {e}"));
        tracing::info!(name, "plan created");
    }

    // ── 3. Seed service catalog ───────────────────────────────────
    tracing::info!("seeding service catalog");

    let services = vec![
        ("auth", "Authentication", "المصادقة", "Core", "lock", true),
        ("settings", "Settings", "الإعدادات", "Core", "settings", true),
        ("audit", "Audit Log", "سجل التدقيق", "Core", "fact_check", true),
        ("projects", "Projects", "المشاريع", "Core", "dashboard", true),
        ("documents", "Documents", "المستندات", "Core", "description", true),
        ("hr", "Human Resources", "الموارد البشرية", "HR", "group", false),
        ("attendance", "Attendance", "الحضور", "HR", "fingerprint", false),
        ("payroll", "Payroll", "الرواتب", "HR", "payments", false),
        ("compliance", "Compliance", "الامتثال", "HR", "gavel", false),
        ("training", "Training", "التدريب", "HR", "school", false),
        ("accounting", "Accounting", "المحاسبة", "Finance", "account_balance", false),
        ("assets", "Assets", "الأصول", "Finance", "apartment", false),
        ("invoicing", "Invoicing", "الفواتير", "Finance", "receipt_long", false),
        ("inventory", "Inventory", "المخزون", "Manufacturing", "inventory_2", false),
        ("manufacturing", "Manufacturing", "التصنيع", "Manufacturing", "precision_manufacturing", false),
        ("crm", "CRM", "إدارة العلاقات", "Sales", "handshake", false),
        ("customer_management", "Customers", "العملاء", "Sales", "people", false),
        ("product_catalog", "Products", "المنتجات", "Sales", "category", false),
        ("store", "B2B Store", "المتجر", "Sales", "storefront", false),
        ("customer_portal", "Customer Portal", "بوابة العملاء", "Sales", "support_agent", false),
        ("ai_agents", "AI Agents", "وكلاء الذكاء الاصطناعي", "Advanced", "smart_toy", false),
        ("iot", "IoT Integration", "إنترنت الأشياء", "Advanced", "sensors", false),
        ("video_training", "Video Training", "التدريب بالفيديو", "Advanced", "videocam", false),
        ("robot_fleet", "Robot Fleet", "أسطول الروبوتات", "Advanced", "precision_manufacturing", false),
        ("hardware_bom", "Hardware BOM", "قائمة المواد", "Advanced", "memory", false),
        ("grants", "Grants & Funding", "المنح والتمويل", "Advanced", "volunteer_activism", false),
        ("energy_monitoring", "Energy Monitoring", "مراقبة الطاقة", "Advanced", "bolt", false),
        ("api_marketplace", "API Marketplace", "سوق الواجهات", "Advanced", "api", false),
        ("field_service", "Field Service", "الخدمة الميدانية", "Operations", "engineering", false),
        ("investor_dashboard", "Investor Dashboard", "لوحة المستثمر", "Operations", "trending_up", false),
        ("enterprise_contracts", "Enterprise Contracts", "العقود المؤسسية", "Operations", "contract", false),
    ];

    for (i, (key, name_en, name_ar, category, icon, is_core)) in services.iter().enumerate() {
        platform_db
            .db
            .query(
                "CREATE service_catalog SET
                    key            = $key,
                    name_en        = $name_en,
                    name_ar        = $name_ar,
                    description_en = $desc_en,
                    description_ar = $desc_ar,
                    icon           = $icon,
                    category       = $category,
                    is_core        = $is_core,
                    sort_order     = $sort_order
                ;",
            )
            .bind(("key", key.to_string()))
            .bind(("name_en", name_en.to_string()))
            .bind(("name_ar", name_ar.to_string()))
            .bind(("desc_en", format!("{name_en} service")))
            .bind(("desc_ar", format!("خدمة {name_ar}")))
            .bind(("icon", icon.to_string()))
            .bind(("category", category.to_string()))
            .bind(("is_core", *is_core))
            .bind(("sort_order", i as i32))
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(key, "service catalog entry: {e}");
                panic!("failed to create service catalog entry '{key}'");
            });
        tracing::info!(key, "service catalog entry created");
    }

    // ── 4. Create super admin account ─────────────────────────────
    tracing::info!("creating super admin account");

    let existing = platform_db.get_account_by_email("admin@actuators.me").await;
    if existing.is_ok() && existing.as_ref().unwrap().is_some() {
        tracing::info!("super admin account already exists, skipping");
    } else {
        let password_hash =
            hash_password("actuators-admin-2026").expect("failed to hash password");
        platform_db
            .db
            .query(
                "CREATE platform_account SET
                    email          = 'admin@actuators.me',
                    password_hash  = $password_hash,
                    full_name      = 'Super Admin',
                    preferred_lang = 'en',
                    is_super_admin = true,
                    email_verified = true,
                    status         = 'active',
                    created_at     = time::now(),
                    updated_at     = time::now()
                ;",
            )
            .bind(("password_hash", password_hash))
            .await
            .expect("failed to create super admin account");
        tracing::info!("super admin account created (admin@actuators.me)");
    }

    tracing::info!("seed complete!");
}
