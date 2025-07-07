use anyhow::Result;
use std::collections::HashMap;
use std::fmt::Debug;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Column {
    pub nullable: bool,
    pub data_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
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

impl<T: Define> Store<T> {
    pub fn new(client: T) -> Self {
        Self {
            schema: HashMap::new(),
            client,
        }
    }
}

pub trait Define {
    type Output;
    async fn get_schema<'a>(&self, schema: &'a str, table: &'a str) -> Result<Self::Output>;
}

impl<T: Define> Define for Store<T> {
    type Output = T::Output;
    async fn get_schema<'a>(&self, schema: &'a str, table: &'a str) -> Result<Self::Output> {
        Ok(self.client.get_schema(schema, table).await?)
    }
}
