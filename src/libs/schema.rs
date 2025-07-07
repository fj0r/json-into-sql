use anyhow::Result;
use std::collections::HashMap;

pub struct Column {
    nullable: bool,
    data_type: String,
}

pub struct Table {
    primary_key: Vec<String>,
    column: HashMap<String, Column>,
}

pub struct Schema {
    table: HashMap<String, Table>,
}

pub struct Store<'a, T: SchemaUpdater> {
    schema: HashMap<String, Schema>,
    client: &'a T
}

pub trait SchemaUpdater {
    async fn get_schema<'a>(&self, schema: &'a str, table: &'a str) -> Result<()>;
}

impl<T: SchemaUpdater> SchemaUpdater for Store<'_, T> {
    async fn get_schema<'a>(&self, schema: &'a str, table: &'a str) -> Result<()> {
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
