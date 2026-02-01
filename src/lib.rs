#![doc = include_str!("../README.md")]
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
    clippy::missing_inline_in_public_items,
    clippy::multiple_crate_versions,
    clippy::error_impl_error,
    reason = "chosen style"
)]
#![expect(clippy::doc_include_without_cfg, reason = "see issue #13918")]

use core::result;
use std::net::TcpStream;

use native_tls::{TlsConnector, TlsStream};

/// An [`imap::Session`] secured through TLS.
type ImapSession = imap::Session<TlsStream<TcpStream>>;

/// A server to connect and interact through IMAP with mailboxes.
#[expect(dead_code, reason = "todo")]
pub struct EmailServer {
    /// Underlying IMAP session of the server that is used for calls to the mailbox.
    session: ImapSession,
}

impl EmailServer {
    /// Creates a new [`EmailServer`] with the given IMAP credentials.
    ///
    /// Their format is specific to each mailbox. For example, for _gmail_, you need an app.
    /// password, and the domain is `imap.gmail.com`.
    ///
    /// If `port` is `None`, it will default to `993`.
    ///
    /// # Errors
    ///
    /// Returns an error if failing to establish the IMAP connection (no credentials, no internet,
    /// etc.).
    pub fn new(
        domain: &str,
        username: &str,
        password: &str,
        port: Option<u16>,
    ) -> Result<Self> {
        let ssl_connector =
            TlsConnector::builder().build().map_err(Error::TlsConnection)?;
        let addr = (domain, port.unwrap_or(993));
        let client = imap::connect(addr, domain, &ssl_connector)
            .map_err(Error::ImapConnection)?;

        let session = client
            .login(username, password)
            .map_err(first)
            .map_err(Error::Login)?;

        Ok(Self { session })
    }
}

/// Custom Result type for this crate.
type Result<T = (), E = Error> = result::Result<T, E>;

/// List of errors that can occur while using a [`EmailServer`]
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to establish the `imap` connection. Domain or port may be incorrect.
    #[error(
        "Failed to establish the `imap` connection. Domain or port may be incorrect."
    )]
    ImapConnection(imap::Error),
    /// Failed to login with the `imap` client. Username or password may be incorrect.
    #[error(
        "Failed to login with the `imap` client. Username or password may be incorrect."
    )]
    Login(imap::Error),
    /// Failed to establish the `native_tls` connection.
    #[error("Failed to establish the `native_tls` connection.")]
    TlsConnection(native_tls::Error),
}

/// Projects a pair on it's first axis.
fn first<T, U>(x: (T, U)) -> T {
    x.0
}
