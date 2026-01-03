use std::{collections::HashMap, path::PathBuf};

use chrono::{Datelike, NaiveDate};
use serde::Deserialize;
use tokio::fs::read;

#[derive(Deserialize)]
#[serde(untagged)]
enum Nicknames {
    One(String),
    Many(Vec<String>),
}

#[derive(Clone, Copy, Default, Deserialize)]
pub(crate) struct Birthday {
    day: u8,
    month: u8,
}

impl Birthday {
    pub fn matches(&self, date: &NaiveDate) -> bool {
        date.day() == self.day as u32 && date.month() == self.month as u32
    }
}

fn birthday_parse<'de, D>(deserializer: D) -> Result<HashMap<String, Birthday>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let map = HashMap::<String, Birthday>::deserialize(deserializer)?;
    Ok(HashMap::from_iter(map.iter().map(|(k, &v)| {
        let mut s = k.clone();
        s.push('s');
        (s.to_lowercase(), v)
    })))
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
        let mut name = n.to_lowercase();
        name.push('s');
        match other_names {
            Nicknames::One(mut nickname) => {
                nickname = nickname.to_lowercase();
                nickname.push('s');
                if inverted.insert(nickname, name).is_some() {
                    return Err(serde::de::Error::custom(format!(
                        "duplicate entry found for: '{n}'"
                    )));
                };
            }
            Nicknames::Many(mut other_names) => {
                other_names.iter_mut().for_each(|s| s.push('s'));
                for nickname in other_names {
                    if inverted
                        .insert(nickname.to_lowercase(), name.clone())
                        .is_some()
                    {
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

fn dir_map<'de, D>(deserializer: D) -> Result<Option<HashMap<String, PathBuf>>, D::Error>
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
    for (mut person, path) in map {
        person.push('s');
        convert.insert(person.to_lowercase(), PathBuf::from(path));
    }

    Ok(Some(convert))
}

#[derive(Clone, Deserialize)]
pub struct Config {
    #[serde(deserialize_with = "birthday_parse")]
    birthdays: HashMap<String, Birthday>,

    #[serde(default, deserialize_with = "invert_map")]
    nicknames: Option<HashMap<String, String>>,

    // a directory containing the code bundle (html + css + js)
    #[serde(default, deserialize_with = "dir_map")]
    path: Option<HashMap<String, PathBuf>>,
}

impl Config {
    pub async fn new() -> Result<Config, Box<dyn std::error::Error>> {
        let path = std::path::Path::new("config.toml");
        let buf = read(path).await?;
        Ok(toml::from_slice(&buf)?)
    }

    pub fn resolve_name<'a>(&'a self, name: &'a str) -> Option<(&'a str, &'a Birthday)> {
        let n = name.split("-").nth(2)?;
        if let Some(birthday) = self.birthdays.get(n) {
            Some((n, birthday))
        } else {
            let nicknames = self.nicknames.as_ref()?;
            Some((nicknames.get(n)?, self.birthdays.get(nicknames.get(n)?)?))
        }
    }

    pub fn resolve_directory(&self, name: &str) -> Option<&PathBuf> {
        self.path.as_ref()?.get(&String::from(name))
    }
}
