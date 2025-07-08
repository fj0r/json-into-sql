use super::postgres::Pg;
use super::schema::Store;
use axum::extract::FromRef;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type PgShared = Arc<RwLock<Store<Pg>>>;

#[derive(Debug, Clone)]
pub struct Shared {
    pub db: Arc<RwLock<Store<Pg>>>,
}

impl FromRef<Shared> for PgShared {
    fn from_ref(input: &Shared) -> Self {
        input.db.clone()
    }
}

impl Shared {
    pub fn new(db: Store<Pg>) -> Self {
        Self {
            db: Arc::new(RwLock::new(db)),
        }
    }
}
