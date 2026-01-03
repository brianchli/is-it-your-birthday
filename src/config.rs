use std::{collections::HashMap, path::PathBuf};

use chrono::{Datelike, NaiveDate};
use serde::Deserialize;
use tokio::fs::read;
use tower_http::services::ServeDir;

#[derive(Deserialize)]
#[serde(untagged)]
enum Nicknames {
    One(String),
    Many(Vec<String>),
}

#[derive(Clone, Default, Deserialize)]
pub(crate) struct Birthday {
    day: u8,
    month: u8,
}

impl Birthday {
    pub fn matches(&self, date: &NaiveDate) -> bool {
        date.day() == self.day as u32 && date.month() == self.month as u32
    }
}

fn invert_map<'de, D>(deserializer: D) -> Result<Option<HashMap<String, String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = Option::<HashMap<String, Nicknames>>::deserialize(deserializer)?;
    let Some(raw) = raw else {
        return Ok(None);
    };

    let mut inverted = HashMap::new();

    for (n, other_names) in raw {
        match other_names {
            Nicknames::One(name) => {
                if inverted.insert(name, n.clone()).is_some() {
                    return Err(serde::de::Error::custom(format!(
                        "duplicate entry found for: '{n}'"
                    )));
                };
            }
            Nicknames::Many(other_names) => {
                for nickname in other_names {
                    if inverted.insert(nickname, n.clone()).is_some() {
                        return Err(serde::de::Error::custom(format!(
                            "duplicate entry found for: '{n}'"
                        )));
                    }
                }
            }
        }
    }

    Ok(Some(inverted))
}

fn servedir<'de, D>(deserializer: D) -> Result<Option<HashMap<String, PathBuf>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // TODO: Harden the path requirement at some point - for now
    // we just assume it exists :)

    let map = Option::<HashMap<String, String>>::deserialize(deserializer)?;
    let Some(map) = map else {
        return Ok(None);
    };

    let mut convert = HashMap::new();
    for (person, path) in map {
        convert.insert(person, PathBuf::from(path));
    }

    Ok(Some(convert))
}

#[derive(Clone, Deserialize)]
pub struct Config {
    birthdays: HashMap<String, Birthday>,

    #[serde(default, deserialize_with = "invert_map")]
    nicknames: Option<HashMap<String, String>>,

    // a directory containing the code bundle (html + css + js)
    #[serde(default, deserialize_with = "servedir")]
    pub path: Option<HashMap<String, PathBuf>>,
}

impl Config {
    pub async fn new() -> Result<Config, Box<dyn std::error::Error>> {
        let path = std::path::Path::new("config.toml");
        let buf = read(path).await?;
        Ok(toml::from_slice(&buf)?)
    }

    pub fn resolve_name<'a>(&'a self, name: &'a String) -> Option<(&'a str, &'a Birthday)> {
        let n = name.as_str().split("-").nth(3)?;
        if let Some(birthday) = self.birthdays.get(n) {
            Some((n, birthday))
        } else {
            let nicknames = self.nicknames.as_ref()?;
            Some((n, self.birthdays.get(nicknames.get(name)?)?))
        }
    }
}
