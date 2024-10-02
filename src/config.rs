use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub debug: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config { debug: true }
    }
}
