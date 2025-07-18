use super::config::JsonType;
use super::error::HttpResult;
use super::schema::{Define, Payload, Table, Val};
use super::shared::{PgShared, Shared};
use crate::libs::error::mkerr;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Map, Value};

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

    // If `allow_list` is None, there are no restrictions.
    if let Some(al) = &db.allow_list {
        let tfn = format!("{}.{}", &schema, &table);
        if !al.contains(&tfn) {
            return mkerr(format!("{} not in allow_list", &tfn));
        };
    };

    let x = db.sync(&schema, &table, &q.force_update).await?;
    Ok(Json(x))
}

#[derive(Deserialize, Debug)]
struct QueryUpsert {
    var: String,
}

fn check_type(typ: &JsonType, v: &Value) -> bool {
    match typ {
        JsonType::I64 => v.is_i64(),
        JsonType::F64 => v.is_f64(),
        JsonType::Str => v.is_string(),
        JsonType::Bool => v.is_boolean(),
        JsonType::Date => v.is_string(),
        JsonType::Unknown => false,
    }
}

async fn upsert(
    Path((schema, table)): Path<(String, String)>,
    Query(q): Query<QueryUpsert>,
    State(db): State<PgShared>,
    Json(data): Json<Value>,
) -> HttpResult<Json<Value>> {
    let db = db.read().await;

    // If `allow_list` is None, there are no restrictions.
    if let Some(al) = &db.allow_list {
        let tfn = format!("{}.{}", &schema, &table);
        if !al.contains(&tfn) {
            return mkerr(format!("{} not in allow_list", &tfn));
        };
    };

    let tbl = db.get(&schema, &table)?;
    if !tbl.variant.contains(&q.var) {
        return mkerr(format!("{} is not a variant", &q.var));
    };

    let d = if data.is_object() {
        &vec![data.clone()]
    } else if data.is_array() {
        data.as_array().unwrap()
    } else {
        &Vec::new()
    };

    let dm = &db.datamap;
    for i in d {
        if let Some(i) = i.as_object() {
            let mut ix = 0;
            let mut columns = Vec::new();
            let mut fields = Vec::new();
            let mut values = Vec::new();
            let mut ext = Map::new();
            for (k, v) in i.iter() {
                if tbl.column.contains_key(k)
                    && let Some(dt) = tbl.column.get(k)
                    && let Some(typ) = dm.get(&dt.data_type)
                    && check_type(typ, &v)
                {
                    ix += 1;
                    fields.push(format!("${}", ix));
                    columns.push(k.to_owned());
                    let val = Val { value: v, typ };
                    values.push(val);
                } else {
                    ext.insert(k.to_string(), v.clone());
                }
            }
            ix += 1;
            fields.push(format!("${}", ix));
            columns.push(q.var.clone());
            let variant = &Value::Object(ext);
            let x = Payload {
                schema: &schema,
                table: &table,
                pk: &tbl.primary_key,
                fields: &fields,
                columns: &columns,
                values,
                variant,
            };
            let _ = db.put(&x).await?;
        };
    }
    Ok(Json(d.len().into()))
}

pub fn data_router() -> Router<Shared> {
    Router::new()
        .route("/list", get(list))
        .route("/schema/{schema}/{table}", get(schema))
        .route("/upsert/{schema}/{table}", post(upsert))
}
