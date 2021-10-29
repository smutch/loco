use anyhow::{format_err, Context, Result};
use colored::Colorize;
use lazy_static::lazy_static;
use serde_derive::Deserialize;
use spinners::{Spinner, Spinners};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use structopt::StructOpt;
use xdg::BaseDirectories;

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

lazy_static! {
    static ref LOCOCONFIG: LocoConfig = LocoConfig::new()
        .context("Failed to parse config.")
        .unwrap();
    static ref LOCOPORT: String = LOCOCONFIG.port.to_string();
}

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(parse(from_os_str), required=true)]
    src: Vec<PathBuf>,
    #[structopt(parse(from_os_str), short, default_value = LOCOCONFIG.dest.to_str().unwrap())]
    dest: PathBuf,
    #[structopt(short, default_value = &LOCOPORT)]
    port: u16,
    #[structopt(short, default_value = &LOCOCONFIG.username)]
    username: String,
    #[structopt(short)]
    verbose: bool,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    // Ensure we have a trailing slash on our destination directory
    let dest = opt.dest.to_str().unwrap();
    let dest = &format!(
        "localhost:{}{}",
        dest,
        if !dest.ends_with('/') { "/" } else { "" }
    );

    // If we don't have a trailing slash in our source then we want to use the '--delete' option to
    // remove files from the destination that shouldn't be there.
    Spinner::new(
        &Spinners::Dots9,
        format!("{}: ", "Grabbing files...".blue()),
    );

    opt.src
        .iter()
        .try_for_each(|src| {
            std::fs::metadata(&src).context(format!(
                "{} {}",
                "Failed to resolve".red(),
                src.to_str().unwrap()
            ))?;

            let source = src.to_str().unwrap();

            let connection = &format!("ssh -p{} -l{}", &opt.port, &opt.username);
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

            match command.status.success() {
                true => {
                    if opt.verbose {
                        println!();
                        std::io::stdout().write_all(command.stdout.as_slice())?;
                    }
                    Ok(())
                }
                false => {
                    std::io::stdout().write_all(command.stdout.as_slice())?;
                    let errors = format!("\n{}\n", String::from_utf8(command.stderr).unwrap());
                    std::io::stderr().write_all(errors.as_bytes())?;
                    Err(format_err!("{}", "Rsync command failed!".red()))
                }
            }
        })
        .map(|res| {
            println!("{}", "...transfer complete".green().bold());
            res
        })
}
