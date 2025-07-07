use anyhow::Result;
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Debug)]
pub struct Column {
    nullable: bool,
    data_type: String,
}

#[derive(Debug)]
pub struct Table {
    primary_key: Vec<String>,
    column: HashMap<String, Column>,
}

#[derive(Debug)]
pub struct Schema {
    table: HashMap<String, Table>,
}

#[derive(Debug)]
pub struct Store<T: Define> {
    schema: HashMap<String, Schema>,
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

impl<T: Define + Debug> Define for Store<T> {
    type Output = ();
    async fn get_schema<'a>(&self, schema: &'a str, table: &'a str) -> Result<Self::Output> {
        self.client.get_schema(schema, table);
        Ok(())
    }
}
