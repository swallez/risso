//use crate::config_rs::*;
use ::config::*;
use std::env;

/// Assemble the configuration from various sources:
/// - built-in defaults
/// - an optional TOML file from the `--config` command line argument
/// - an optional `local.toml` file, for local development overrides
pub fn load() -> Result<Config, ConfigError> {
    let mut s = Config::new();

    // Load defaults
    s.merge(File::from_str(include_str!("defaults.toml"), FileFormat::Toml))?;

    // Find an optional "--config" command-line argument

    // Poor man's args parsing. Ok since we only have one, otherwise use the clap crate
    let mut args = env::args();
    while let Some(arg) = args.next() {
        if arg == "--config" {
            break;
        }
    }

    if let Some(path) = args.next() {
        s.merge(File::with_name(&path))?;
    }

    // Load an optional local file (useful for development)
    s.merge(File::with_name("local").required(false))?;

    //s.try_into()
    Ok(s)
}
