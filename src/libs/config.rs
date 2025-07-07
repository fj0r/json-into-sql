use crate::concat_fields;
use figment::{
    Figment, Result,
    providers::{Env, Format, Toml},
};
use serde::Deserialize;

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

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Config {
    pub database: Database,
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
