//! A simple CLI to test the application with your own mailbox.

#![warn(
    missing_docs,
    warnings,
    deprecated_safe,
    future_incompatible,
    keyword_idents,
    let_underscore,
    nonstandard_style,
    refining_impl_trait,
    rust_2018_compatibility,
    rust_2018_idioms,
    rust_2021_compatibility,
    rust_2024_compatibility,
    unused,
    clippy::all,
    clippy::pedantic,
    clippy::style,
    clippy::perf,
    clippy::complexity,
    clippy::correctness,
    clippy::restriction,
    clippy::nursery,
    clippy::cargo
)]
#![allow(
    clippy::single_call_fn,
    clippy::implicit_return,
    clippy::pattern_type_mismatch,
    clippy::blanket_clippy_restriction_lints,
    clippy::missing_trait_methods,
    clippy::question_mark_used,
    clippy::mod_module_files,
    clippy::module_name_repetitions,
    clippy::pub_with_shorthand,
    clippy::unseparated_literal_suffix,
    clippy::else_if_without_else,
    reason = "chosen style"
)]

use std::env;

use color_eyre::eyre::Context as _;
use dotenv::dotenv;

/// Custom Result for this example.
type Result<T = ()> = color_eyre::Result<T>;

/// Loads an environment variable with a nicer error if not present.
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
