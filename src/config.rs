use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub debug: bool,
    pub verbose_debug: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            debug: true,
            verbose_debug: false,
        }
    }
}
