use super::config::AllowList;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub nullable: bool,
    pub data_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub primary_key: Vec<String>,
    pub column: HashMap<String, Column>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Schema {
    pub table: HashMap<String, Table>,
}

#[derive(Debug)]
pub struct Store<T: Define> {
    pub schema: HashMap<String, Schema>,
    pub allow_list: AllowList,
    pub client: T,
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub schema: String,
    pub table: String,
    pub content: Table,
}

impl<T: Define> Store<T> {
    pub fn new(client: T, allow_list: AllowList) -> Self {
        Self {
            schema: HashMap::new(),
            allow_list,
            client,
        }
    }
    pub fn update(&mut self, entity: Entity) -> Result<()> {
        let mut t = if let Some(s) = self.schema.remove(&entity.schema) {
            s
        } else {
            let table = HashMap::new();
            Schema { table }
        };
        t.table.insert(entity.table, entity.content);
        self.schema.insert(entity.schema, t);
        Ok(())
    }
}

pub trait Define {
    type Output;
    async fn sync<'a>(&mut self, schema: &'a str, table: &'a str) -> Result<Self::Output>;
}
