use anyhow::{Context, Result};
use serde_derive::Deserialize;
use std::path::PathBuf;
use std::process::Command;
use structopt::StructOpt;
use xdg::BaseDirectories;
use spinners::{Spinner, Spinners};
use std::io::Write;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(parse(from_os_str))]
    path: PathBuf,
    dest: Option<PathBuf>,
    port: Option<u16>,
    username: Option<String>,
}

#[derive(Deserialize, Debug)]
struct LocoConfig {
    port: u16,
    dest: PathBuf,
    username: String,
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
    std::fs::metadata(&opt.path).context("Failed to resolve source path")?;

    let loco_config = LocoConfig::new()
        .context("Failed to parse config.")?;

    // TODO: Try out some macro magic for this...
    let dest = opt.dest.unwrap_or(loco_config.dest);
    let port = opt.port.unwrap_or(loco_config.port);
    let username = opt.username.unwrap_or(loco_config.username);

    let dest = dest.to_str().unwrap();
    let spinner = Spinner::new(&Spinners::Dots9, format!("Making a local copy of {}\n", dest));
    
    // Ensure we have a trailing slash on our destination directory
    let dest = &format!(
        "localhost:{}{}",
        dest,
        if !dest.ends_with('/') {
            "/"
        } else {
            ""
        }
    );

    // If we don't have a trailing slash in our source then we want to use the '--delete' option to
    // remove files from the destination that shouldn't be there.
    let source = opt.path.to_str().unwrap();

    let connection = &format!("-e 'ssh -p{} -l{}'", &port, &username);
    let mut args = vec!["-r", "-n", connection];

    if !source.ends_with('/') {
        args.push("--delete");
    }

    args.push(source);
    args.push(dest);

    println!("{:?}", args);
    let command = Command::new("rsync").args(&args).output().context("Failed to launch rsync command")?;

    std::io::stdout().write_all(command.stdout.as_slice())?;
    std::io::stderr().write_all(command.stderr.as_slice())?;


    spinner.stop();

    Ok(())
}
