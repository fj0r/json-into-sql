use super::config::Database;
use super::schema::{Define, SchemaUpdater, Store};
use anyhow::Result;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions, query};

pub async fn conn(config: &Database) -> Result<Pool<Postgres>> {
    let c: String = config.to_url();
    let pool = PgPoolOptions::new().max_connections(5).connect(&c).await?;
    Ok(pool)
}

impl SchemaUpdater for Pool<Postgres> {
    async fn get_schema<'a>(&self, schema: &'a str, table: &'a str) -> Result<()> {
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
        Ok(())
    }
}

impl Define for Store<'_, Pool<Postgres>> {}
