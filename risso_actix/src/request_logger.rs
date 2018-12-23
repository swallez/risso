use slog;

use risso_api::logs::macros::*;

use actix_web::{Error, FromRequest, HttpRequest};
use actix_web_requestid::RequestIDGetter;
use std::ops::Deref;

/// A `FromRequest` integrating `slog` with the `actix_request_id` crate: it resolves to a
/// `slog::Logger` that has a `request_id` key/value pair to allow tracing a request in the log
/// statements of code that contributed to processing it.
///
/// The `scope()` method runs a closure in the context of the resquest logger.

pub struct RequestLogger(slog::Logger);

impl RequestLogger {
    /// Deconstruct to an inner value
    pub fn into_inner(self) -> slog::Logger {
        self.0
    }

    /// Execute code in the request's logging scope. Convenience wrapper around `slog_scope::scope()`.
    #[inline]
    pub fn scope<SF, R>(&self, f: SF) -> R
    where
        SF: FnOnce() -> R,
    {
        slog_scope::scope(&self.0, f)
    }
}

impl<S> FromRequest<S> for RequestLogger {
    type Config = ();
    type Result = Result<Self, Error>;

    #[inline]
    fn from_request(req: &HttpRequest<S>, _: &Self::Config) -> Self::Result {
        // String processing because request_id.0 is private
        let req_id = format!("{:?}", req.request_id())
            .replace("RequestID(\"", "")
            .replace("\")", "");

        // Return the current logger augmented with the request_id
        let new_log = slog_scope::logger().new(slog_o!("request_id" => req_id));

        Ok(RequestLogger(new_log))
    }
}

/// Allow direct access to `Logger` methods from a `RequestLogger`.
///
impl Deref for RequestLogger {
    type Target = slog::Logger;

    fn deref(&self) -> &slog::Logger {
        &self.0
    }
}
