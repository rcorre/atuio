use anyhow::{bail, Result};
use atuio::{config::Config, tui};
use clap::Parser;

pub const APP_NAME: &'static str = env!("CARGO_PKG_NAME");

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct CLI {
    path: Option<std::path::PathBuf>,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = CLI::parse();

    let xdg = xdg::BaseDirectories::with_prefix(APP_NAME)?;

    let config_path = xdg.get_config_file("config.toml");
    log::debug!("Reading config from {config_path:?}");

    let config = match std::fs::read_to_string(&config_path) {
        Ok(s) => {
            log::trace!("Read config:\n {s:?}");
            toml::from_str(&s)?
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            log::trace!("Using default config");
            Config::default()
        }
        Err(err) => {
            bail!("Failed to read {config_path:?}: {err:?}");
        }
    };

    log::trace!("Using config:\n {config:?}");

    tui::start(config, args.path)
}
