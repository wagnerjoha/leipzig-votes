pub mod muni_geo;
pub mod votes_lpz;

use std::collections::HashMap;
use std::fs;

use anyhow::{Context, Error};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub vote_sources: Vec<VoteSource>,
}

#[derive(Deserialize, Debug)]
pub struct VoteSource {
    pub url: String,
    pub name: String,
    pub party_map: HashMap<String, String>,
}

pub fn load_config() -> Result<Config, Error> {
    let config_str = fs::read_to_string("harvester.toml").context("Failed to read config file")?;
    let config: Config = toml::from_str(&config_str).context("Failed to parse config file")?;
    Ok(config)
}
