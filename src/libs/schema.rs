use super::config::{AllowList, DataMap, JsonType};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use super::redis::RedisPool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub nullable: bool,
    pub data_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub primary_key: Vec<String>,
    pub variant: HashSet<String>,
    pub column: HashMap<String, Column>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Schema {
    pub table: HashMap<String, Table>,
}

#[derive(Debug)]
pub(crate) struct Store<T> {
    pub schema: HashMap<String, Schema>,
    pub datamap: DataMap,
    pub allow_list: AllowList,
    pub client: T,
    pub redis: RedisPool,
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub schema: String,
    pub table: String,
    pub content: Table,
}

#[derive(Debug, Clone)]
pub struct Val<'a> {
    pub value: &'a Value,
    pub typ: &'a JsonType,
}

#[derive(Debug, Clone)]
pub struct Payload<'a> {
    pub schema: &'a String,
    pub table: &'a String,
    pub columns: &'a Vec<String>,
    pub fields: &'a Vec<String>,
    pub values: Vec<Val<'a>>,
    pub pk: &'a Vec<String>,
    pub variant: &'a Value,
}

impl<T> Store<T> {
    pub fn new(client: T, allow_list: AllowList, datamap: DataMap, redis: RedisPool) -> Self {
        Self {
            datamap,
            schema: HashMap::new(),
            allow_list,
            client,
            redis,
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
    async fn sync<'a>(
        &mut self,
        schema: &'a str,
        table: &'a str,
        force: &'a Option<bool>,
    ) -> Result<Self::Output>;
    fn get<'a>(&self, schema: &'a str, table: &'a str) -> Result<Self::Output>;
    async fn put<'a>(&self, payload: &Payload<'a>) -> Result<()>;
}
