use std::{collections::HashMap, path::PathBuf};

use super::Birthday;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub(crate) enum Actions {
    #[serde(alias = "redirect")]
    #[serde(alias = "to")]
    Redirect(String),
    #[serde(untagged)]
    Resolve(String),
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum Single {
    Text(String),
    Table(Actions),
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum Aliases {
    Single(Single),
    Multiple(Vec<Single>),
}

pub fn birthday_parse<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, (Actions, Birthday)>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let map = HashMap::<String, Birthday>::deserialize(deserializer)?;
    Ok(HashMap::from_iter(map.iter().map(|(k, &v)| {
        let mut s = k.to_lowercase();
        s.push('s');
        (s.clone(), (Actions::Resolve(s), v))
    })))
}

pub fn invert_map<'de, D>(deserializer: D) -> Result<Option<HashMap<String, Actions>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let map = Option::<HashMap<String, Aliases>>::deserialize(deserializer)?;
    let Some(map) = map else {
        return Ok(None);
    };

    let mut inverted = HashMap::new();
    let mut handle_inversion = |name: String, single: Single| -> Result<(), D::Error> {
        match single {
            Single::Text(mut target) => {
                target.push('s');
                if let Some(duplicate) = inverted.insert(target, Actions::Resolve(name)) {
                    return Err(serde::de::Error::custom(format!(
                        "Duplicate found {:?}",
                        duplicate
                    )));
                };
                Ok(())
            }
            Single::Table(redirect) => {
                let mut redirect = match redirect {
                    Actions::Redirect(name) => name,
                    Actions::Resolve(_) => unreachable!(
                        "no resolves can be created as an object within the configuration"
                    ),
                };
                redirect.push('s');
                if let Some(duplicate) = inverted.insert(redirect, Actions::Redirect(name)) {
                    return Err(serde::de::Error::custom(format!(
                        "Duplicate found {:?}",
                        duplicate
                    )));
                };
                Ok(())
            }
        }
    };
    for (mut original, mappings) in map {
        original.push('s');
        original.chars().for_each(|mut c| c.make_ascii_lowercase());
        match mappings {
            Aliases::Single(single) => handle_inversion(original, single)?,
            Aliases::Multiple(singles) => {
                for s in singles {
                    handle_inversion(original.clone(), s)?;
                }
            }
        }
    }

    Ok(Some(inverted))
}

pub fn dir_map<'de, D>(deserializer: D) -> Result<Option<HashMap<String, PathBuf>>, D::Error>
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
