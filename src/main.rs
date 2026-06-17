mod config;
mod errors;

use anyhow::Result;
use config::RulixConfig;

fn main() -> Result<()> {
    let config_path = "local/config.yaml";

    let config = RulixConfig::from_file(config_path)?;
    println!("{:#?}", config);

    Ok(())
}
