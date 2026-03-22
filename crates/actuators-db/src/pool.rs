use std::num::NonZeroUsize;
use std::sync::Mutex;

use actuators_core::config::AppConfig;
use actuators_core::error::AppError;
use lru::LruCache;
use surrealdb::engine::any::Any;
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tracing::{debug, instrument};

use crate::tenant::TenantDb;

/// An LRU-based pool of per-tenant SurrealDB connections.
///
/// Each tenant has its own database within the shared SurrealDB namespace.
/// The pool lazily creates connections on first access and evicts the
/// least-recently-used connection when the cache is full.
pub struct TenantConnectionPool {
    connections: Mutex<LruCache<String, Surreal<Any>>>,
    config: AppConfig,
}

impl TenantConnectionPool {
    /// Create a new pool sized according to `config.tenant_pool_max_size`.
    pub fn new(config: AppConfig) -> Self {
        let max_size = NonZeroUsize::new(config.tenant_pool_max_size)
            .unwrap_or(NonZeroUsize::new(50).unwrap());

        Self {
            connections: Mutex::new(LruCache::new(max_size)),
            config,
        }
    }

    /// Return the configured maximum pool size.
    pub fn max_size(&self) -> usize {
        self.config.tenant_pool_max_size
    }

    /// Obtain a [`TenantDb`] connected to the given tenant database.
    ///
    /// If the connection is already cached it is returned immediately;
    /// otherwise a new WebSocket connection is established, authenticated,
    /// switched to the correct namespace + db, cached, and returned.
    #[instrument(skip(self))]
    pub async fn get(&self, db_name: &str) -> Result<TenantDb, AppError> {
        // Step 1: Check cache under the lock.
        {
            let mut cache = self.connections.lock().map_err(|e| {
                AppError::Internal(format!("connection pool lock poisoned: {e}"))
            })?;

            if let Some(conn) = cache.get(db_name) {
                debug!(db_name, "tenant connection cache hit");
                return Ok(TenantDb::new(conn.clone()));
            }
        }
        // Lock is dropped here so we can do async work.

        // Step 2: Create a new connection (async).
        debug!(db_name, "tenant connection cache miss — connecting");
        let conn = self.connect_tenant(db_name).await?;

        // Step 3: Re-acquire the lock and insert.
        {
            let mut cache = self.connections.lock().map_err(|e| {
                AppError::Internal(format!("connection pool lock poisoned: {e}"))
            })?;

            // Another task may have inserted while we were connecting.
            // Overwrite is fine — the LRU keeps the most recent.
            cache.put(db_name.to_owned(), conn.clone());
        }

        Ok(TenantDb::new(conn))
    }

    /// Obtain a raw `Surreal<Any>` client for the given tenant database.
    /// Useful for provisioning and other low-level work that doesn't need
    /// the `TenantDb` wrapper.
    #[instrument(skip(self))]
    pub async fn get_raw(&self, db_name: &str) -> Result<Surreal<Any>, AppError> {
        // Step 1: Check cache.
        {
            let mut cache = self.connections.lock().map_err(|e| {
                AppError::Internal(format!("connection pool lock poisoned: {e}"))
            })?;

            if let Some(conn) = cache.get(db_name) {
                debug!(db_name, "tenant connection cache hit (raw)");
                return Ok(conn.clone());
            }
        }

        // Step 2: Create connection.
        debug!(db_name, "tenant connection cache miss (raw) — connecting");
        let conn = self.connect_tenant(db_name).await?;

        // Step 3: Cache and return.
        {
            let mut cache = self.connections.lock().map_err(|e| {
                AppError::Internal(format!("connection pool lock poisoned: {e}"))
            })?;
            cache.put(db_name.to_owned(), conn.clone());
        }

        Ok(conn)
    }

    /// Establish a fresh connection to SurrealDB for the given tenant database.
    async fn connect_tenant(&self, db_name: &str) -> Result<Surreal<Any>, AppError> {
        let db = surrealdb::engine::any::connect(&self.config.surrealdb_url)
            .await
            .map_err(|e| {
                AppError::Database(format!("failed to connect for tenant {db_name}: {e}"))
            })?;

        db.signin(Root {
            username: self.config.surrealdb_user.clone(),
            password: self.config.surrealdb_pass.clone(),
        })
        .await
        .map_err(|e| {
            AppError::Database(format!("failed to sign in for tenant {db_name}: {e}"))
        })?;

        db.use_ns(&self.config.surrealdb_namespace)
            .use_db(db_name)
            .await
            .map_err(|e| {
                AppError::Database(format!(
                    "failed to select ns/db for tenant {db_name}: {e}"
                ))
            })?;

        Ok(db)
    }
}
