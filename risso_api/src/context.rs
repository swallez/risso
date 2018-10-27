#![allow(dead_code, deprecated)]

// Republish diesel's manager so that server impls don't have to add it to their deps.
pub use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;

use futures::future::Future;

#[derive(Deserialize)]
struct ContextConfig {
    db_path: String,
    min_connections: u32,
    max_connections: u32,
}

/// Single location where choose the actual database backend we're using.
/// TODO: add compile-time feature to choose between SQLite, PG and MySQL
///
pub type Connection = diesel::sqlite::SqliteConnection;
pub type DB = diesel::sqlite::Sqlite;

/// Base type from which `ApiContext` objects can be built. It must be held by the main thread, as it
/// contains the main thread pool object from which individual `Executors` can be obtained.
/// Dropping `ApiContextBootstrap` drops the pool.
///
pub struct ApiBuilder {
    pub cnx_pool: Pool<ConnectionManager<Connection>>,
    pub thread_pool: tokio_threadpool::ThreadPool,
    pub registry: prometheus::Registry,
}

impl ApiBuilder {
    pub fn new() -> Result<ApiBuilder, failure::Error> {
        let config = super::CONFIG.get::<ContextConfig>("database")?;

        info!(
            "Using database at {} with max {} connections.",
            config.db_path, config.max_connections
        );

        let cnx_manager = ConnectionManager::<Connection>::new(config.db_path);

        let cnx_pool = Pool::builder()
            .max_size(config.max_connections)
            .min_idle(Some(config.min_connections))
            .build(cnx_manager)?;

        let thread_pool = tokio_threadpool::Builder::new()
            .name_prefix("risso-api")
            .keep_alive(Some(std::time::Duration::from_secs(30)))
            .pool_size(config.max_connections as usize)
            .build();

        let registry = prometheus::Registry::new();

        Ok(ApiBuilder {
            cnx_pool,
            thread_pool,
            registry,
        })
    }

    pub fn build(&self) -> ApiContext {
        ApiContext {
            cnx_pool: self.cnx_pool.clone(),
            executor: self.thread_pool.sender().clone(),
        }
    }
}

#[derive(Clone)]
pub struct ApiContext {
    cnx_pool: Pool<ConnectionManager<Connection>>,
    executor: tokio_threadpool::Sender,
}

impl ApiContext {
    // https://github.com/diesel-rs/diesel/issues/399#issuecomment-360535059

    /// Run a blocking operation on the database on the context's thread pool and return a future
    pub fn spawn_db<F, T, E>(&self, f: F) -> impl Future<Item = T, Error = failure::Error>
    where
        T: Send + 'static,
        E: std::error::Error + Send + Sync + 'static,
        F: FnOnce(&Connection) -> Result<T, E> + Send + 'static,
    {
        use futures::sync::oneshot;

        let pool = self.cnx_pool.clone();
        oneshot::spawn_fn(
            move || {
                let cnx = pool.get().map_err(failure::Error::from)?;
                f(&cnx).map_err(failure::Error::from)
            },
            &self.executor,
        )
    }
}
