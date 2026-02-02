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
#![expect(clippy::print_stdout, clippy::print_stderr, reason = "this is a cli")]

use std::env::{self, VarError};
use std::io::{Write as _, stdin, stdout};
use std::path::PathBuf;

use color_eyre::eyre::{Context as _, ContextCompat as _};
use dotenv::dotenv;
use e_mail::EmailServer;

/// Custom Result for this example.
type Result<T = ()> = color_eyre::Result<T>;

/// Loads an environment variable if present, otherwise prompts the user for it.
fn env(var_name: &str) -> Result<String> {
    match env::var(var_name) {
        Ok(var) => return Ok(var),
        Err(VarError::NotUnicode(_)) => {
            eprintln!(
                "{var_name} was found but contains unsupported characters"
            );
        }
        Err(VarError::NotPresent) => (),
    }
    print!("Enter {var_name}:");
    stdout().flush()?;
    let value = stdin().lines().next().context("Failed to read stdin")??;
    Ok(value)
}

/// Loads the .env file if it exists.
fn load_env() -> Result {
    let env_file = PathBuf::from(".env");
    if env_file.is_file() {
        dotenv().context("Failed to load `.env` file")?;
    } else if env_file.exists() {
        eprintln!(".env exists but is not a file, skipping.");
    }
    Ok(())
}

/// Determines the credentials and uses them to connect an [`EmailServer`].
fn login_server() -> Result<EmailServer> {
    let domain = env("DOMAIN")?;
    let username = env("USERNAME")?;
    let password = env("PASSWORD")?;
    let port = env::var("PORT")
        .ok()
        .map(|port| {
            port.parse().with_context(|| {
                format!("Failed to parse {port} into a port number")
            })
        })
        .transpose()?;

    Ok(EmailServer::new(&domain, &username, &password, port)?)
}

/// Writes a prompt
fn prompt() -> Result {
    print!("\x1b[33m> \x1b[0m");
    stdout().flush()?;
    Ok(())
}

fn main() -> Result {
    color_eyre::install()?;
    load_env()?;
    let mut server = login_server()?;

    prompt()?;
    for next_line in stdin().lines() {
        let line = next_line.context("Failed to read input")?;

        if line == "list_mailboxes" {
            let mailboxes =
                server.list_mailboxes().context("Failed to list mailboxes")?;
            for mailbox_name in mailboxes {
                println!("{mailbox_name}");
            }
        } else {
            println!("Invalid command {line}");
        }
        prompt()?;
    }

    Ok(())
}
