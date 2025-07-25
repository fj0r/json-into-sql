use super::config::{Database, JsonType};
use super::schema::{Column, Define, Entity, Payload, Store, Table, Val};
use anyhow::{Result, anyhow};
use futures::TryStreamExt;
use sqlx::{Pool, Postgres, Row, postgres::PgPoolOptions, query};
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use tracing::info;

pub async fn conn(config: &Database) -> Result<Pool<Postgres>> {
    let c: String = config.to_url();
    let pool = PgPoolOptions::new().max_connections(5).connect(&c).await?;
    Ok(pool)
}

#[derive(Debug)]
pub struct Pg(pub Pool<Postgres>);

impl Deref for Pg {
    type Target = Pool<Postgres>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Pg {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Pg {
    async fn upsert<'a>(&self, pl: &Payload<'a>) -> Result<()> {
        let cs = pl.columns.join(", ");
        let ecs = pl
            .columns
            .iter()
            .map(|x| format!("excluded.{}", x))
            .collect::<Vec<_>>()
            .join(", ");
        let fs = pl.fields.join(", ");
        let pks = pl.pk.join(", ");
        let tn = format!("{}.{}", pl.schema, pl.table);
        let sql = format!(
            r#"
            INSERT INTO {} ({}) VALUES ({})
            ON CONFLICT ({})
            DO UPDATE
            SET ({}) = ({})
        "#,
            tn, cs, fs, pks, cs, ecs
        );
        let mut x = pl.values.clone();
        x.push(Val {
            value: pl.variant,
            typ: &JsonType::Unknown,
        });
        let mut r = query(&sql);
        for i in x {
            match i.typ {
                JsonType::I64 => {
                    r = r.bind(i.value.as_i64().unwrap());
                }
                JsonType::F64 => {
                    r = r.bind(i.value.as_f64().unwrap());
                }
                JsonType::Str => {
                    r = r.bind(i.value.as_str().unwrap());
                }
                JsonType::Bool => {
                    r = r.bind(i.value.as_bool().unwrap());
                }
                JsonType::Date => {
                    let v = i.value.as_str().unwrap();
                    r = r.bind(v);
                }
                JsonType::Unknown => {
                    r = r.bind(i.value);
                }
            };
        }
        let mut r = r.fetch(&**self);

        while let Some(i) = r.try_next().await? {
            println!("{:?}", i);
        }
        Ok(())
    }
    async fn fetch<'a>(&self, schema: &'a str, table: &'a str) -> Result<Entity> {
        let mut x =
        query(r#"
            WITH ct AS (
                SELECT ccu.table_schema, ccu.table_name, ccu.column_name, tc.constraint_type IS NOT NULL AS pk
                FROM information_schema.table_constraints AS tc
                JOIN information_schema.constraint_column_usage AS ccu
                ON tc.constraint_schema = ccu.constraint_schema
                    AND tc.constraint_name = ccu.constraint_name
                WHERE tc.constraint_type = 'PRIMARY KEY'
            ) SELECT co.column_name, co.is_nullable, co.data_type, COALESCE(ct.pk, false) AS pk
            FROM information_schema.columns AS co
            LEFT OUTER JOIN ct
            ON co.table_schema = ct.table_schema
              AND co.table_name = ct.table_name
              AND co.column_name = ct.column_name
            WHERE co.table_schema = $1
              AND co.table_name = $2
         "#)
        .bind(schema)
        .bind(table)
        .fetch(&**self);

        let mut primary_key = Vec::new();
        let mut variant = HashSet::new();
        let mut column = HashMap::new();
        while let Some(r) = x.try_next().await? {
            let name: &str = r.try_get("column_name")?;
            let data_type: &str = r.try_get("data_type")?;
            if data_type == "jsonb" {
                variant.insert(name.to_owned());
            };
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
                primary_key.push(name.to_owned());
            }
        }
        Ok(Entity {
            schema: schema.to_string(),
            table: table.to_string(),
            content: Table {
                primary_key,
                variant,
                column,
            },
        })
    }
}

impl Define for Store<Pg> {
    type Output = Table;
    async fn sync<'a>(
        &mut self,
        schema: &'a str,
        table: &'a str,
        force: &'a Option<bool>,
    ) -> Result<Self::Output> {
        let force = force.unwrap_or(false);
        if !force
            && let Some(s) = self.schema.get(schema)
            && let Some(t) = s.table.get(table)
        {
            Ok(t.clone())
        } else {
            info!("sync schema from {}.{}", schema, table);
            let r = self.client.fetch(schema, table).await?;
            self.update(r.clone())?;
            Ok(r.content)
        }
    }

    fn get<'a>(&self, schema: &'a str, table: &'a str) -> Result<Self::Output> {
        if let Some(s) = self.schema.get(schema)
            && let Some(t) = s.table.get(table)
        {
            Ok(t.to_owned())
        } else {
            Err(anyhow!("not fount"))
        }
    }
    async fn put<'a>(&self, payload: &Payload<'a>) -> Result<()> {
        let _ = self.client.upsert(payload).await?;
        Ok(())
    }
}
