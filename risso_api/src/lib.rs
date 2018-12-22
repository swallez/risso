#![allow(dead_code, deprecated)]

// Some crates still need pre-2018 macro_use either because their macros are private or
// because of name mismatch.
#[macro_use]
extern crate prometheus;
#[macro_use]
extern crate validator_derive;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate lazy_static;

use serde_derive::{Deserialize, Serialize};

//use config as config_rs; // rename it as we have our own 'config' module

use chrono::prelude::*;

use futures::future::Future;

use crate::context::ApiContext;

use validator::Validate;

mod config;
pub mod context;
pub mod dieselext;
pub mod log_macros;
pub mod models;
pub mod schema;

lazy_static! {
    /// Global configuration object. Each module can pick its own section in the configuration.
    pub static ref CONFIG: ::config::Config = crate::config::load_config().unwrap();

    /// General configurations (private to this crate)
    static ref GENERAL_CONFIG: GeneralConfig = CONFIG.get("general").unwrap();

    /// General configurations (private to this crate)
    static ref SMTP_CONFIG: SmtpConfig = CONFIG.get("smtp").unwrap();

}

// newtype: use defer to pull wrapped type's methods
// https://doc.rust-lang.org/book/second-edition/ch19-03-advanced-traits.html#using-the-newtype-pattern-to-implement-external-traits-on-external-types

pub type ThreadId = i32;
pub type CommentId = i32;

#[derive(Deserialize)]
struct GeneralConfig {
    gravatar_url: String,
}

#[derive(Deserialize)]
struct SmtpConfig {
    username: Option<String>,
    password: Option<String>,
    host: String,
    port: i32,
    to: String,
    from: String,
}

/// A boxed future returning a generic result and a `failure::Error`. Shortcut to simplify return statements.
/// Boxed futures allow returning various Future implementations from a function.
type BoxFuture<T> = Box<Future<Item = T, Error = failure::Error>>;

/// Validate an object that implement `validator::Validate` and return a boxed future error
/// if validation failed.
///
/// Typical usage is `if let Some(err) = validate(&v) { return err; }` or using the `validate!` macro.
///
pub fn validate<T: validator::Validate, U: 'static + Send>(v: &T) -> Option<BoxFuture<U>> {
    if let Err(e) = v.validate() {
        Some(futures::failed(e.into()).boxed())
    } else {
        None
    }
}

/// Convenience macro to validate a set of objects that implement `validator::Validate` and return
/// a boxed future error if validation failed.
///
/// Usage: `validate!(foo, bar, baz);`
///
macro_rules! validate {
    ( $( $x:expr ),* ) => {
        $( if let Some(e) = validate(&$x) { return e; } )*
    }
}

//--------------------------------------------------------------------------------------------------
// Common api structures

#[derive(Serialize)]
pub struct CommentResponse {
    id: CommentId,
    parent: Option<i32>,
    text: String,
    author: Option<String>,
    website: Option<String>,
    mode: i32,
    created: DateTime<Utc>,
    modified: Option<DateTime<Utc>>,
    likes: i32,
    dislikes: i32,
    hash: String,
    gravatar_image: String,
}

//--------------------------------------------------------------------------------------------------
// New comment

#[derive(Clone, Deserialize, Validate)]
pub struct NewComment {
    author: Option<String>,
    email: Option<String>,
    text: String,
    parent: Option<i32>,
    website: Option<String>,
}

pub fn new_comment(_ctx: &ApiContext, _uri: String, req: NewComment) -> BoxFuture<CommentResponse> {
    validate!(req);

    unimplemented!()
}

/// Sanitize html
///
/// ```rust
/// use risso_api::sanitize_html;
///
/// assert_eq!(sanitize_html("foo"), "foo".to_owned());
/// ```
///
pub fn sanitize_html(html: &str) -> String {
    // See https://posativ.org/isso/docs/configuration/server/#markup

    let mut sanitizer = ammonia::Builder::default();

    sanitizer.add_tags(
        vec![
            "a",
            "blockquote",
            "br",
            "code",
            "del",
            "em",
            "h1",
            "h2",
            "h3",
            "h4",
            "h5",
            "h6",
            "hr",
            "img",
            "ins",
            "li",
            "ol",
            "p",
            "pre",
            "strong",
            "table",
            "tbody",
            "td",
            "th",
            "thead",
            "ul",
        ]
        .into_iter(),
    );

    sanitizer.clean(html).to_string()
}

pub fn send_new_comment_email(title: &str, _comment: &NewComment) -> Result<(), failure::Error> {
    use lettre::*;
    use lettre_email::EmailBuilder;
    use native_tls::TlsConnector;

    let email = EmailBuilder::new()
        .from(SMTP_CONFIG.from.clone())
        .to(SMTP_CONFIG.to.clone())
        .subject(format!("New comment on {}", title))
        .text("foo")
        .build()?;

    let tls_parameters = ClientTlsParameters::new(String::from("foo"), TlsConnector::builder()?.build()?);
    let mut mailer =
        //SmtpTransport::builder_unencrypted_localhost()?.build();
        SmtpTransport::builder("blah", ClientSecurity::Wrapper(tls_parameters))?.build();

    mailer.send(&email).map(|_| ()).map_err(|err| err.into())
}

//--------------------------------------------------------------------------------------------------
// Fetch

#[derive(Clone, Debug, Deserialize, Validate)]
pub struct FetchRequest {
    /// The URI of the thread to gets comments from.
    #[validate(length(max = "1024"))]
    uri: String,
    parent: Option<CommentId>,
    limit: Option<i64>,
    nested_limit: Option<usize>,
    after: Option<DateTime<Utc>>,
    plain: Option<i32>,
}

impl FetchRequest {
    pub fn is_plain(&self) -> bool {
        // As defined in the Isso docs
        self.plain.unwrap_or(0) == 1
    }
}

#[derive(Serialize)]
pub struct FetchResponse {
    id: i32,
    total_replies: i32,
    hidden_replies: i32,
    replies: Vec<CommentResponse>,
}

pub fn fetch(ctx: &ApiContext, req: FetchRequest) -> BoxFuture<Vec<CommentResponse>> {
    validate!(req);

    let _root_id = req.parent;
    let plain = req.is_plain();

    let after: f64 = req.after.map_or(0.0f64, |date| dieselext::FloatDateTime(date).to_f64());

    let req1 = req.clone();
    let reply_counts = ctx.spawn_db(move |cnx| models::Comment::reply_count(cnx, req1.uri, None, after));

    // comments
    let root_list =
        ctx.spawn_db(move |cnx| models::Comment::fetch(cnx, req.uri, None, after, req.parent, None, true, req.limit));

    reply_counts
        .join(root_list)
        .map(move |(_reply_counts, root_list)| process_fetched_list(&root_list, plain))
        .boxed()
}

fn process_fetched_list(list: &Vec<models::Comment>, plain: bool) -> Vec<CommentResponse> {
    list.iter()
        .map(|item| {
            let mut digest = sha1::Sha1::new();
            digest.update(item.email.as_ref().unwrap_or(&item.remote_addr).as_bytes());

            // Fallback on ip-address for the gravatar, for a somewhat stable image
            let email_md5 = format!("{:x}", md5::compute(item.email.as_ref().unwrap_or(&item.remote_addr)));
            let gravatar_image = GENERAL_CONFIG.gravatar_url.replace("{}", &email_md5);

            let text = if plain {
                item.text.clone()
            } else {
                let md_parser = pulldown_cmark::Parser::new(&item.text);
                let mut html = String::new();
                pulldown_cmark::html::push_html(&mut html, md_parser);
                html
            };

            CommentResponse {
                id: item.id,
                parent: item.parent,
                text,
                author: item.author.clone(),
                website: item.website.clone(),
                mode: item.mode,
                created: item.created.0,
                modified: item.modified.map(|d| d.0),
                likes: item.likes,
                dislikes: item.dislikes,

                hash: digest.digest().to_string(),
                gravatar_image,
            }
        })
        .collect()
}

//--------------------------------------------------------------------------------------------------
// Unsubscribe

pub fn unsubscribe2(_ctx: &ApiContext) -> BoxFuture<CommentResponse> {
    futures::done(Err(failure::err_msg("Not implemented yet")).into()).boxed()
}

pub fn unsubscribe(_ctx: &ApiContext, _id: String, _email: String, _key: String) -> BoxFuture<()> {
    futures::done(Err(failure::err_msg("Not implemented yet")).into()).boxed()
}

#[cfg(test)]
mod tests {
    use futures::Future;
    use validator::Validate;

    #[derive(Validate)]
    struct Address {
        #[validate(email)]
        email: String,
    }

    #[test]
    fn validate_should_succeed() {
        let addr = Address {
            email: String::from("foo@bar.com"),
        };

        assert!(super::validate::<Address, ()>(&addr).is_none());
    }

    #[test]
    fn validate_should_fail() {
        let addr = Address {
            email: String::from("foo"),
        };

        let opt_future = super::validate::<Address, ()>(&addr);
        assert!(opt_future.is_some());

        let result = opt_future.unwrap().wait();
        assert!(result.is_err());
    }

    #[test]
    #[should_panic]
    fn demonstrate_should_panic() {
        let x: Option<String> = None;

        x.unwrap();
    }
}
