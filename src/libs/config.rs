use crate::concat_fields;
use figment::{
    Figment, Result,
    providers::{Env, Format, Toml},
};
use serde::{Deserialize, de::Visitor};
use std::{collections::{HashMap, HashSet}, ops::Deref};

pub type AllowList = Option<HashSet<String>>;

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Database {
    #[serde(rename = "type")]
    pub kind: String,
    pub host: String,
    pub port: u16,
    pub db: String,
    pub schema: Option<String>,
    pub user: String,
    pub passwd: String,
    pub allow_list: AllowList,
}

impl Database {
    #[allow(dead_code)]
    pub fn to_st(self: &Database) -> String {
        concat_fields! {
            var self;
            host;
            port;
            dbname = db;
            user;
            password = passwd;
        }
    }
    #[allow(dead_code)]
    pub fn to_url(self: &Database) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user, self.passwd, self.host, self.port, self.db
        )
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub enum LogFormat {
    #[allow(non_camel_case_types)]
    json,
    #[default]
    #[allow(non_camel_case_types)]
    compact,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Log {
    pub format: LogFormat,
}

#[derive(Debug, Clone)]
pub(crate) enum JsonType {
    I64,
    F64,
    Str,
    Bool,
    Date,
    Unknown
}

#[derive(Debug, Clone)]
pub(crate) struct DataMap(HashMap<String, JsonType>);

impl<'de> Deserialize<'de> for DataMap {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct DataMapVistor;
        impl<'de> Visitor<'de> for DataMapVistor {
            type Value = DataMap;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct DataMap")
            }
            fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut r = HashMap::new();
                while let Some((key, value)) = map.next_entry::<String, Vec<String>>()? {
                    let k = match key.as_str() {
                        "integer" => JsonType::I64,
                        "float" => JsonType::F64,
                        "string" => JsonType::Str,
                        "bool" => JsonType::Bool,
                        "date" => JsonType::Date,
                        _ => JsonType::Unknown
                    };
                    for v in value {
                        r.insert(v, k.clone());
                    }
                }
                Ok(DataMap(r))
            }
        }
        const FIELDS: &[&str] = &["0"];
        deserializer.deserialize_struct("DataMap", FIELDS, DataMapVistor)
    }
}

impl Deref for DataMap {
    type Target = HashMap<String, JsonType>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Config {
    pub database: Database,
    pub datamap: DataMap,
    pub trace: Log,
}

impl Config {
    pub fn new() -> Result<Self> {
        Figment::new()
            .merge(Toml::file("config.toml"))
            .merge(Env::prefixed("CONFIG_").split("_"))
            .extract()
    }
}
