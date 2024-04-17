use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub line_numbers: bool, 
}

impl Default for Config {
    fn default() -> Self {
        Self {
            line_numbers: true,
        }
    }
}
