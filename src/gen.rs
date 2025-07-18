use serde::Deserialize;
use std::collections::HashMap;
use anyhow::Result;

#[derive(Debug, Deserialize)]
enum Kind {
    #[serde(rename = "type")]
    literal(String),
    references(Vec<String>),
    #[serde(rename = "enum")]
    variant(Vec<String>),
}

#[derive(Debug, Deserialize)]
struct Field {
    #[serde(flatten, rename = "type")]
    kind: Kind,
    default: Option<String>,
    #[serde(default)]
    uniq: bool,
    #[serde(default)]
    notnull: bool
}

#[derive(Debug, Deserialize)]
struct Table {
    fields: HashMap<String, Field>,
    primary: Option<Vec<String>>,
    index: Option<Vec<String>>,
}


fn main() -> Result<()> {
    let f = std::env::args().nth(1).and_then(|x| {
        std::fs::read_to_string(x).ok()
    });
    if let Some(f) = f {
        let x: HashMap<String, Table> = toml::from_str(&f).unwrap();
        println!("{:#?}", &x);
    }

    Ok(())
}
