use anyhow::{format_err, Context, Result};
use colored::Colorize;
use serde_derive::Deserialize;
use spinners::{Spinner, Spinners};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use structopt::StructOpt;
use xdg::BaseDirectories;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(parse(from_os_str))]
    path: PathBuf,
    dest: Option<PathBuf>,
    port: Option<u16>,
    username: Option<String>,
    verbose: Option<bool>,
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

    let loco_config = LocoConfig::new().context("Failed to parse config.")?;

    // TODO: Try out some macro magic for this...
    let dest = opt.dest.unwrap_or(loco_config.dest);
    let port = opt.port.unwrap_or(loco_config.port);
    let username = opt.username.unwrap_or(loco_config.username);

    let dest = dest.to_str().unwrap();
    let spinner = Spinner::new(
        &Spinners::Dots9,
        format!("{} {}\n", "Making a local copy of".blue(), dest.blue()),
    );

    // Ensure we have a trailing slash on our destination directory
    let dest = &format!(
        "localhost:{}{}",
        dest,
        if !dest.ends_with('/') { "/" } else { "" }
    );

    // If we don't have a trailing slash in our source then we want to use the '--delete' option to
    // remove files from the destination that shouldn't be there.
    let source = opt.path.to_str().unwrap();

    let connection = &format!("ssh -p{} -l{}", &port, &username);
    let mut args = vec!["-r", "-n", "-e", connection];

    if !source.ends_with('/') {
        args.push("--delete");
    }

    args.push(source);
    args.push(dest);

    let command = Command::new("rsync")
        .args(&args)
        .output()
        .context("Failed to launch rsync command")?;

    spinner.stop();

    match command.status.success() {
        true => {
            if opt.verbose.unwrap_or(false) {
                println!();
                std::io::stdout().write_all(command.stdout.as_slice())?;
            }
            Ok(())
        }
        false => {
            std::io::stdout().write_all(command.stdout.as_slice())?;
            println!();
            let errors = format!("{}", String::from_utf8(command.stderr).unwrap().red());
            std::io::stderr().write_all(errors.as_bytes())?;
            println!();
            Err(format_err!("Rsync command failed!"))
        }
    }
}
