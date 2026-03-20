// ملف اختباري بسيط - للتحقق من اتصال قاعدة البيانات
// NOTE: هذا الملف يستخدم main مستقل - لتشغيله: cargo run --bin test_db

use Actuators::config::AppConfig;
use Actuators::db::{AppState, StatsCache};
use Actuators::i18n::I18n;
use std::collections::HashMap;
use std::sync::Arc;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tokio::sync::{RwLock, broadcast};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let config = AppConfig::from_env().expect("Failed to load config");

    let (board_tx, _) = broadcast::channel(100);

    let db: Surreal<Client> = Surreal::new::<Ws>(&config.db.url)
        .await
        .expect("Failed to connect to SurrealDB");

    db.signin(Root {
        username: &config.db.user,
        password: &config.db.pass,
    })
    .await
    .expect("Failed to sign in");

    db.use_ns(&config.db.namespace)
        .use_db(&config.db.database)
        .await
        .expect("Failed to select namespace/database");

    let state = AppState {
        db,
        jwt_secret: config.jwt.secret.clone(),
        jwt_expiry_hours: config.jwt.expiry_hours,
        i18n: Arc::new(I18n::new()),
        board_events: board_tx,
        stats_cache: Arc::new(RwLock::new(StatsCache {
            data: HashMap::new(),
            last_updated: std::time::Instant::now(),
        })),
    };

    println!("Testing clients...");
    match Actuators::domains::customers::repository::get_all_clients(&state).await {
        Ok(clients) => println!("Clients loaded: {}", clients.len()),
        Err(e) => println!("Clients Error: {:?}", e),
    }

    println!("Testing departments...");
    match Actuators::domains::hr_org::repository::get_all_departments(&state).await {
        Ok(deps) => println!("Departments loaded: {}", deps.len()),
        Err(e) => println!("Departments Error: {:?}", e),
    }
}
