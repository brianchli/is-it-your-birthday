use crate::core::parse::{Actions, Birthday, birthday_parse, dir_map, invert_map};
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf};
use tokio::fs::read;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(deserialize_with = "birthday_parse")]
    pub(crate) birthdays: HashMap<String, (Actions, Birthday)>,

    #[serde(default, deserialize_with = "invert_map")]
    pub(crate) aliases: Option<HashMap<String, Actions>>,

    // a directory containing the code bundle (html + css + js)
    #[serde(default, deserialize_with = "dir_map")]
    pub(crate) path: Option<HashMap<String, PathBuf>>,
}

impl Config {
    pub async fn new() -> Result<Config, Box<dyn std::error::Error>> {
        let path = std::path::Path::new("config.toml");
        let buf = read(path).await?;
        Ok(toml::from_slice(&buf)?)
    }
}
