use anyhow::Result;
use std::collections::HashMap;

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
pub struct Store<T: SchemaUpdater> {
    schema: HashMap<String, Schema>,
    pub client: T,
}

impl<T: SchemaUpdater> Store<T> {
    pub fn new(client: T) -> Self {
        Self {
            schema: HashMap::new(),
            client,
        }
    }
}

pub trait SchemaUpdater {
    type Output;
    async fn get_schema<'a>(&self, schema: &'a str, table: &'a str) -> Result<Self::Output>;
}

impl<T: SchemaUpdater> SchemaUpdater for Store<T> {
    type Output = ();
    async fn get_schema<'a>(&self, schema: &'a str, table: &'a str) -> Result<Self::Output> {
        self.client.get_schema(schema, table);
        Ok(())
    }
}

pub trait Define: SchemaUpdater {
    async fn update(&mut self, schema: &str, table: &str) -> Result<()> {
        self.get_schema(schema, table);
        Ok(())
    }
}
