use anyhow::Result;
use serde::Deserialize;
use std::collections::BTreeMap as Map;
use std::collections::VecDeque;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
enum Typ {
    #[allow(non_camel_case_types)]
    #[serde(rename = "type")]
    literal(String),
    #[allow(non_camel_case_types)]
    references((String, String)),
    #[allow(non_camel_case_types)]
    #[serde(rename = "enum")]
    variant(Vec<String>),
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Field {
    #[serde(flatten, rename = "type")]
    typ: Typ,
    default: Option<String>,
    #[serde(default)]
    uniq: bool,
    #[serde(default)]
    notnull: bool,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Index {
    column: Vec<String>,
    #[serde(rename = "type")]
    typ: Option<String>,
    include: Option<Vec<String>>,
    with: Option<Map<String, String>>,
    #[serde(rename = "where")]
    where_: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Table {
    column: Map<String, Field>,
    primary: Option<Vec<String>>,
    index: Option<Vec<Index>>,
}

struct Cols {
    name: String,
    typ: String,
    x: VecDeque<String>,
    refs: Option<String>,
    enu: Option<Vec<String>>,
}

impl std::fmt::Display for Cols {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut x = self.x.clone();
        if let Some(r) = &self.refs {
            x.push_back(r.to_string())
        }
        x.push_front(self.typ.clone());
        x.push_front(self.name.clone());
        write!(f, "    {}", x.as_slices().0.join(" "))
    }
}

fn gen_columns(table: &String, name: &String, field: &Field, schema: &Map<String, Table>) -> Cols {
    let mut enu = None;
    let mut refs = None;
    let mut x = VecDeque::new();
    let typ = match &field.typ {
        Typ::literal(l) => l.to_uppercase(),
        Typ::references((table, column)) => {
            let t = schema.get(table).unwrap();
            let c = t.column.get(column).unwrap();
            refs = Some(format!("REFERENCES {} ({})", table, column));
            match &c.typ {
                Typ::literal(x) => match x.as_str() {
                    "serial" => "integer",
                    "bigserial" => "biginteger",
                    _ => x,
                }
                .to_uppercase(),
                _ => unreachable!(),
            }
        }
        Typ::variant(v) => {
            enu = Some(v.clone());
            format!("{}_{}", table, name)
        }
    };
    if field.notnull {
        x.push_back("NOT NULL".to_string());
    }
    if let Some(d) = &field.default {
        let v = match typ.as_str() {
            "JSONB" => format!("'{}'::JSONB", d),
            _ => d.to_owned(),
        };
        x.push_back(format!("DEFAULT {}", v));
    }
    if field.uniq {
        x.push_back("UNIQ".to_string());
    }

    Cols {
        name: name.to_string(),
        typ,
        x,
        refs,
        enu,
    }
}

fn gen_table(schema: &Map<String, Table>) -> Vec<String> {
    let mut r = Vec::new();
    for (k, v) in schema.iter() {
        let mut cs = Vec::new();
        for (l, w) in v.column.iter() {
            let x = gen_columns(k, l, w, schema);
            if let Some(en) = &x.enu {
                let m: Vec<_> = en.into_iter().map(|x| format!("'{x}'")).collect();
                let m = m.join(", ");
                r.push(format!("CREATE TYPE {} AS ENUM ({});", x.typ, m))
            }
            cs.push(x.to_string());
        }
        if let Some(p) = &v.primary {
            cs.push(format!("    PRIMARY KEY ({})", p.join(", ")))
        }
        let cs = cs.join(",\n");
        r.push(format!("CREATE TABLE {} (\n{}\n);", k, cs));
        if let Some(ixs) = &v.index {
            for ix in ixs {
                let mut rest = Vec::new();
                if let Some(typ) = &ix.typ {
                    rest.push(format!("USING {}", typ.to_uppercase()));
                }
                rest.push(format!("({})", ix.column.join(", ")));
                if let Some(inc) = &ix.include {
                    rest.push(format!("INCLUDE ({})", inc.join(", ")));
                }
                if let Some(with) = &ix.with {
                    let w = with
                        .iter()
                        .fold(Vec::new(), |mut acc, (k, v)| {
                            acc.push(format!("{} = {}", k, v));
                            acc
                        })
                        .join(", ");
                    rest.push(format!("WITH ({})", w));
                }
                if let Some(w) = &ix.where_ {
                    rest.push(format!("WHERE {}", w));
                }
                let name = format!("idx_{}_{}", k, ix.column.join("_"));
                r.push(format!("CREATE INDEX {} on {} {};", name, k, rest.join(" ")));
            }
        }
    }
    r
}

fn main() -> Result<()> {
    let f = std::env::args()
        .nth(1)
        .and_then(|x| std::fs::read_to_string(x).ok());
    if let Some(f) = f {
        let x: Map<String, Table> = toml::from_str(&f).unwrap();
        println!("{:#?}", &x);
        for s in gen_table(&x) {
            println!("{}", &s);
        }
    }
    Ok(())
}
