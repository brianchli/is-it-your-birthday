use std::path::PathBuf;

use crate::core::config::Config;
use crate::core::parse::{Actions, Birthday};

pub struct Handler;
impl Handler {
    pub fn execute<'a>(
        Config {
            birthdays,
            aliases,
            path,
        }: &'a Config,
        name: &str,
    ) -> Option<(&'a Actions, Option<&'a PathBuf>, Option<&'a Birthday>)> {
        let n = name.split("-").nth(2)?;
        if let Some((action, birthday)) = birthdays.get(n) {
            let p = n.to_string();
            if let Some(pathbufs) = path {
                let path = pathbufs.get(&p);
                Some((action, path, Some(birthday)))
            } else {
                Some((action, None, Some(birthday)))
            }
        } else {
            let aliases = aliases.as_ref()?;
            let alias = aliases.get(n)?;
            match alias {
                Actions::Redirect { .. } => Some((alias, None, None)),
                Actions::Resolve(to) => Some((
                    alias,
                    path.as_ref()?.get(to),
                    birthdays.get(to).map(|(_, birthday)| birthday),
                )),
            }
        }
    }
}
