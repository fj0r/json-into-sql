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
    pub client: T,
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub schema: String,
    pub table: String,
    pub content: Table,
}

impl<T: Define> Store<T> {
    pub fn new(client: T) -> Self {
        Self {
            schema: HashMap::new(),
            client,
        }
    }
    pub fn update(&mut self, entity: Entity) -> Result<()> {
        self.schema
            .entry(entity.schema)
            .and_modify(|s| {
                s.table
                    .entry(entity.table.clone())
                    .and_modify(|t| {
                        *t = entity.content.clone();
                    })
                    .or_insert_with(|| entity.content.clone());
            })
            .or_insert_with(|| {
                let mut table = HashMap::new();
                table.insert(entity.table, entity.content);
                Schema { table }
            });
        Ok(())
    }
}

pub trait Define {
    type Output;
    async fn sync<'a>(&mut self, schema: &'a str, table: &'a str) -> Result<Self::Output>;
}
