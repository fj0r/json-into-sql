use super::error::HttpResult;
use super::shared::{Pg, Shared};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, post},
};

use futures::TryStreamExt;
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::{Row, query};
use std::collections::HashMap;
use std::ops::Deref;

async fn schema(
    State(db): State<Pg>,
    Path((schema, table)): Path<(String, String)>,
) -> HttpResult<Json<Vec<Value>>> {
    let db = db.read().await;
    println!("{}, {}", &schema, &table);
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
        .fetch(db.deref());
    let mut v = Vec::new();
    while let Some(r) = x.try_next().await? {
        dbg!(&r);
    }
    Ok(Json(v)).into()
}

#[derive(Deserialize, Debug)]
struct QueryParams {
    var: String,
}

async fn upsert(
    Path((schema, table)): Path<(String, String)>,
    Query(q): Query<QueryParams>,
    State(db): State<Pg>,
    Json(data): Json<Value>,
) -> HttpResult<Json<Value>> {
    let db = db.read().await;
    println!("{}, {}", schema, table);
    println!("{}", data);
    println!("{:?}", q);
    Ok(Json(data))
}

pub fn data_router() -> Router<Shared> {
    Router::new()
        .route("/schema/{schema}/{table}", get(schema))
        .route("/upsert/{schema}/{table}", post(upsert))
}
