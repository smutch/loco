use anyhow::{Context, Result};
use serde_derive::Deserialize;
use std::path::PathBuf;
use structopt::StructOpt;
use xdg::BaseDirectories;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(parse(from_os_str))]
    path: PathBuf,
    port: Option<u16>,
}

#[derive(Deserialize, Debug)]
struct LocoConfig {
    port: u16,
}

impl LocoConfig {
    fn new() -> Result<Self, config::ConfigError> {
        let mut settings = config::Config::default();
        let config_dir = BaseDirectories::with_prefix("loco").unwrap();

        settings.merge(
            config::File::with_name(config_dir.get_config_file("config").to_str().unwrap())
                .required(false),
        )?;
        settings.merge(config::Environment::with_prefix("loco"))?;

        settings.try_into()
    }
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    std::fs::metadata(opt.path).context("Failed to resolve source path")?;

    let loco_config = LocoConfig::new().context("Failed to parse config. Ensure either LOCO_PORT environment variable is set or '$XDG_CONFIG/loco/config.*' exists.");

    Ok(())
}
