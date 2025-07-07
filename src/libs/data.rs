use super::error::HttpResult;
use super::shared::{Pg, Shared};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, post},
};

use super::schema::{Define, Table};
use serde::Deserialize;
use serde_json::Value;

async fn schema(
    State(db): State<Pg>,
    Path((schema, table)): Path<(String, String)>,
) -> HttpResult<Json<Table>> {
    let mut db = db.write().await;
    let x = db.sync(&schema, &table).await?;
    Ok(Json(x))
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
    println!("{:?}", q.var);
    println!("{:?}", db);
    Ok(Json(data))
}

pub fn data_router() -> Router<Shared> {
    Router::new()
        .route("/schema/{schema}/{table}", get(schema))
        .route("/upsert/{schema}/{table}", post(upsert))
}
