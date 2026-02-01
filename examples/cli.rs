use std::env;

use color_eyre::eyre::Context as _;
use dotenv::dotenv;

type Result<T = ()> = color_eyre::Result<T>;

fn env(var_name: &str) -> Result<String> {
    env::var(var_name).with_context(|| {
        format!("Failed to load {var_name} environment variable. Consider adding it to the `.env` file.")
    })
}

fn main() -> Result {
    color_eyre::install()?;
    dotenv().context("Failed to load `.env` file. Please create it with the DOMAIN, USERNAME and PASSWORD variables.")?;
    let _domain = env("DOMAIN");
    let _username = env("USERNAME");
    let _password = env("PASSWORD");
    Ok(())
}
