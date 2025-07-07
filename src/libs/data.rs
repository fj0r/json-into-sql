use super::error::HttpResult;
use super::shared::{Pg, Shared};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, post},
};

use super::schema::SchemaUpdater;
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
    let x = db.deref().get_schema(&schema, &table).await?;
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
