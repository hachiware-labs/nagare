use std::collections::HashMap;
use std::path::PathBuf;

use nagare_core::resolve_root;

#[derive(Debug)]
pub(crate) struct ParsedArgs {
    pub(crate) options: HashMap<String, String>,
    pub(crate) positionals: Vec<String>,
}

impl ParsedArgs {
    pub(crate) fn parse(args: &[String]) -> Result<Self, String> {
        let mut options = HashMap::new();
        let mut positionals = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            if arg.starts_with("--") {
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| format!("{arg} requires a value"))?;
                options.insert(arg.clone(), value.clone());
            } else {
                positionals.push(arg.clone());
            }
            i += 1;
        }
        Ok(Self {
            options,
            positionals,
        })
    }

    pub(crate) fn root(&self) -> Result<PathBuf, String> {
        resolve_root(self.optional("--root")).map_err(|error| error.to_string())
    }

    pub(crate) fn required(&self, name: &str) -> Result<&str, String> {
        self.optional(name)
            .ok_or_else(|| format!("{name} is required"))
    }

    pub(crate) fn optional(&self, name: &str) -> Option<&str> {
        self.options.get(name).map(String::as_str)
    }
}
