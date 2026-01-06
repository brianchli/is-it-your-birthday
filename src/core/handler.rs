use std::path::PathBuf;

use crate::core::config::{Actions, Birthday, Config};

pub struct Handler;
impl Handler {
    pub fn execute<'a>(
        Config {
            birthdays,
            aliases,
            path,
        }: &'a Config,
        name: &str,
    ) -> Option<(Actions, Option<&'a PathBuf>, Option<&'a Birthday>)> {
        let n = name.split("-").nth(2)?;
        if let Some(birthday) = birthdays.get(n) {
            let p = n.to_string();
            if let Some(pathbufs) = path {
                let path = pathbufs.get(&p);
                Some((Actions::Resolve(p), path, Some(birthday)))
            } else {
                Some((Actions::Resolve(p), None, Some(birthday)))
            }
        } else {
            let aliases = aliases.as_ref()?;
            let alias = aliases.get(n)?;
            match alias {
                Actions::Redirect { .. } => Some((alias.clone(), None, None)),
                Actions::Resolve(to) => {
                    Some((alias.clone(), path.as_ref()?.get(to), birthdays.get(to)))
                }
            }
        }
    }
}
