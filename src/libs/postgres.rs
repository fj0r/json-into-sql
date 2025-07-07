use super::config::Database;
use super::schema::{Column, Define, Store, Table};
use anyhow::Result;
use futures::TryStreamExt;
use sqlx::{Pool, Postgres, Row, postgres::PgPoolOptions, query};
use std::collections::HashMap;

pub async fn conn(config: &Database) -> Result<Pool<Postgres>> {
    let c: String = config.to_url();
    let pool = PgPoolOptions::new().max_connections(5).connect(&c).await?;
    Ok(pool)
}

impl Define for Pool<Postgres> {
    type Output = Table;
    async fn get_schema<'a>(&self, schema: &'a str, table: &'a str) -> Result<Self::Output> {
        let mut x =
        query(r#"
            with ct as (
                select ccu.table_schema, ccu.table_name, ccu.column_name, tc.constraint_type is not null as pk
                from information_schema.table_constraints as tc
                join information_schema.constraint_column_usage as ccu
                on tc.constraint_schema = ccu.constraint_schema
                    and tc.constraint_name = ccu.constraint_name
                where tc.constraint_type = 'PRIMARY KEY'
            ) select co.column_name, co.is_nullable, co.data_type, coalesce(ct.pk, false) as pk
            from information_schema.columns as co
            left join ct
            on co.table_schema = ct.table_schema
              and co.table_name = ct.table_name
              and co.column_name = ct.column_name
            where co.table_schema = $1
              and co.table_name = $2
         "#)
        .bind(schema)
        .bind(table)
        .fetch(self);

        let mut pks = Vec::new();
        let mut column = HashMap::new();
        while let Some(r) = x.try_next().await? {
            let name: &str = r.try_get("column_name")?;
            let data_type: &str = r.try_get("data_type")?;
            let data_type: String = data_type.to_owned();
            let nullable: &str = r.try_get("is_nullable")?;
            let nullable = nullable == "YES";
            column.insert(
                name.to_owned(),
                Column {
                    nullable,
                    data_type,
                },
            );
            let pk: bool = r.try_get("pk")?;
            if pk {
                pks.push(name.to_owned());
            }
        }
        Ok(Table {
            primary_key: pks,
            column,
        })
    }
}

