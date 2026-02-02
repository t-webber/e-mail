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
use std::collections::{BTreeMap, HashMap};
use std::net::TcpStream;

use native_tls::{TlsConnector, TlsStream};
use proc_macros::doc_error;
use utf7_imap::decode_utf7_imap;

/// Convient macro to create an IMAP error closure with a formatted message.
///
/// # Examples
///
/// ```
/// let formatted = "data";
/// let e: Result<(), imap::Error>;
/// e.map_err(imap_err!("Some {formatted} string"));
/// ```
macro_rules! imap_err {
    ($($arg:tt)*) => {
        |err| Error::ImapError(format!($($arg),*), err)
    };
}

/// An [`imap::Session`] secured through TLS.
type ImapSession = imap::Session<TlsStream<TcpStream>>;

/// A server to connect and interact through IMAP with mailboxes.
pub struct EmailServer {
    /// Caches the list of mailboxes to not refetch everytime.
    mailbox_names: Vec<String>,
    /// Caches the data of each mailbox
    mailboxes: HashMap<String, MailboxData>,
    /// Underlying IMAP session of the server that is used for calls to the mailbox.
    session: ImapSession,
}

impl EmailServer {
    /// Returns the list of the names of the mailboxes (i.e., folders) that exist for the current session.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let mut server = e_mail::EmailServer::new("imap.gmail.com", "my.email@gmail.com", "sixteenletterkey",
    /// None).unwrap();
    /// for mailbox_name in server.list_mailboxes() {
    ///    println!("{mailbox_name}");
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// It returns an error if it failed to prompt the server for the list (internet error, etc.)
    #[must_use]
    pub fn list_mailboxes(&self) -> &[String] {
        &self.mailbox_names
    }

    /// Creates a new [`EmailServer`] with the given IMAP credentials.
    ///
    /// Their format is specific to each mailbox. For example, for _gmail_, you need an app.
    /// password, and the domain is `imap.gmail.com`.
    ///
    /// If `port` is `None`, it will default to `993`.
    ///
    /// # Examples
    ///
    /// ```
    /// e_mail::EmailServer::new("imap.gmail.com", "my.email@gmail.com", "sixteenletterkey", None);
    /// ```
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
        let ssl_connector = TlsConnector::builder().build()?;
        let addr = (domain, port.unwrap_or(993));
        let client = imap::connect(addr, domain, &ssl_connector)
            .map_err(Error::ImapConnection)?;

        let session = client
            .login(username, password)
            .map_err(first)
            .map_err(Error::Login)?;

        let mut this =
            Self { mailboxes: HashMap::new(), mailbox_names: vec![], session };
        this.refresh()?;
        Ok(this)
    }

    /// Refetches everything to make sure all data is up-to-date.
    ///
    /// # Errors
    ///
    /// Will fail if network is down.
    pub fn refresh(&mut self) -> Result {
        self.mailbox_names = list_mailboxes(&mut self.session)?;
        for mailbox_name in &self.mailbox_names {
            self.session
                .select(mailbox_name)
                .map_err(imap_err!("select {mailbox_name}"))?;
            if let Some(mailbox) = self.mailboxes.get_mut(mailbox_name) {
                mailbox.refresh(&mut self.session)?;
            } else {
                let mailbox_data = MailboxData::new(&mut self.session)?;
                self.mailboxes.insert(mailbox_name.to_owned(), mailbox_data);
            }
            self.session.close().map_err(imap_err!("close {mailbox_name}"))?;
        }
        Ok(())
    }
}

/// All data of an email
struct EmailData;

/// All data of a mailbox, cached here.
struct MailboxData(BTreeMap<u32, EmailData>);

impl MailboxData {
    /// Populates the data of a new Mailbox
    fn new(session: &mut ImapSession) -> Result<Self> {
        Ok(Self(
            session
                .uid_search("ALL")
                .map_err(imap_err!("uid_search(ALL)"))?
                .into_iter()
                .map(|uid| (uid, EmailData))
                .collect(),
        ))
    }

    /// Re-populates the data of a mailbox with the additional data received.
    fn refresh(&mut self, session: &mut ImapSession) -> Result {
        session
            .uid_search("ALL")
            .map_err(imap_err!("uid_search(ALL)"))?
            .into_iter()
            .for_each(|uid| {
                self.0.entry(uid).or_insert_with(|| EmailData);
            });
        Ok(())
    }
}

/// Custom Result type for this crate.
type Result<T = (), E = Error> = result::Result<T, E>;

/// List of errors that can occur while using a [`EmailServer`]
#[non_exhaustive]
#[doc_error]
pub enum Error {
    #[error("Failed to connect with IMAP. Check domain, port and network.")]
    ImapConnection(#[source] imap::Error),
    #[error("Failed to run '{0}' on the IMAP session.")]
    ImapError(String, #[source] imap::Error),
    #[error(
        "Failed to login with the IMAP client. Check username and password."
    )]
    Login(#[source] imap::Error),
    #[error("Failed to establish the `native_tls` connection.")]
    TlsConnection(#[from] native_tls::Error),
}

/// Projects a pair on it's first axis.
fn first<T, U>(x: (T, U)) -> T {
    x.0
}

/// Lists all the mailboxes of an [`ImapSession`]
fn list_mailboxes(session: &mut ImapSession) -> Result<Vec<String>> {
    Ok(session
        .list(None, Some("*"))
        .map_err(imap_err!("list(None,*)"))?
        .into_iter()
        .filter(|mailbox| {
            session.select(mailbox.name()).and_then(|_| session.close()).is_ok()
        })
        .map(|mailbox| decode_utf7_imap(mailbox.name().to_owned()))
        .collect())
}
