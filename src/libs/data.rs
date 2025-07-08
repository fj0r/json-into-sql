use crate::libs::error::mkerr;

use super::error::HttpResult;
use super::schema::{Define, Table};
use super::shared::{PgShared, Shared};
use anyhow::anyhow;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::Value;

async fn list(State(db): State<PgShared>) -> HttpResult<Json<Vec<String>>> {
    let db = db.read().await;
    let mut r = Vec::new();
    for (k, v) in &db.schema {
        for (l, _w) in &v.table {
            r.push(format!("{}.{}", k, l));
        }
    }
    Ok(Json(r))
}

#[derive(Deserialize, Debug)]
struct QuerySchema {
    force_update: Option<bool>,
}
async fn schema(
    Query(q): Query<QuerySchema>,
    Path((schema, table)): Path<(String, String)>,
    State(db): State<PgShared>,
) -> HttpResult<Json<Table>> {
    let mut db = db.write().await;
    let x = db.sync(&schema, &table, &q.force_update).await?;
    Ok(Json(x))
}

#[derive(Deserialize, Debug)]
struct QueryUpsert {
    var: String,
}

async fn upsert(
    Path((schema, table)): Path<(String, String)>,
    Query(q): Query<QueryUpsert>,
    State(db): State<PgShared>,
    Json(data): Json<Value>,
) -> HttpResult<Json<Value>> {
    let db = db.read().await;
    let tbl = db.get(&schema, &table)?;
    if !tbl.variant.contains(&q.var) {
        return mkerr(format!("{} is not a variant", &q.var));
    };
    println!("{:?}", tbl);
    println!("{}, {}", schema, table);
    println!("{}", data);
    println!("{:?}", q.var);
    println!("{:?}", db);
    Ok(Json(data))
}

pub fn data_router() -> Router<Shared> {
    Router::new()
        .route("/list", get(list))
        .route("/schema/{schema}/{table}", get(schema))
        .route("/upsert/{schema}/{table}", post(upsert))
}
