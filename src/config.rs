use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub option: ConfigOption,
    #[serde(default)]
    pub verilog: ConfigVerilog,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigOption {
    #[serde(default = "default_as_true")]
    pub linter: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigVerilog {
    #[serde(default)]
    pub include_paths: Vec<PathBuf>,
    #[serde(default)]
    pub defines: Vec<String>,
    #[serde(default)]
    pub plugins: Vec<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        toml::from_str("").unwrap()
    }
}

impl Default for ConfigOption {
    fn default() -> Self {
        toml::from_str("").unwrap()
    }
}

impl Default for ConfigVerilog {
    fn default() -> Self {
        toml::from_str("").unwrap()
    }
}

#[allow(dead_code)]
fn default_as_true() -> bool {
    true
}

#[allow(dead_code)]
fn default_as_false() -> bool {
    false
}
