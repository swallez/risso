use log;
use slog;
use slog_async;
use slog_json;
use slog_scope;
use slog_term;

/// Setup the slog logging framework and the log->slog bridge for crates that use log.
///
/// The result contains the root logger and a slog-scope global logger guard for this root logger.
/// The global logger is unset once the guard is dropped.
///
pub fn setup_slog() -> (slog_scope::GlobalLoggerGuard, slog::Logger) {
    use slog::*;

    let decorator = slog_term::TermDecorator::new().force_color().build();
    let _term_drain = slog_term::FullFormat::new(decorator).build();

    let _json_drain = slog_json::Json::default(std::io::stderr());

    // Pick your format
    //let drain = _term_drain;
    let drain = _json_drain;

    // Display only info+
    let drain = drain.filter_level(Level::Info);

    let drain = slog_async::Async::new(drain.fuse()).build().fuse();

    let log = Logger::root(
        drain,
        slog_o!(
            "location" => FnValue(|info : &Record| {
                format!("{}:{}", info.module(), info.line())
            })
        ),
    );

    // Bridge std log
    log::set_boxed_logger(Box::new(SlogStdLogger(log.clone()))).unwrap();
    log::set_max_level(log::LevelFilter::max());

    // Set slog default logger
    let guard = slog_scope::set_global_logger(log.clone());

    (guard, log)
}

/// Bridge from log to slog. The slog-stdlog crate has not yet been updated to log 0.4
/// (see https://github.com/slog-rs/stdlog/pull/5)
///
struct SlogStdLogger(slog::Logger);

impl SlogStdLogger {
    #[inline]
    pub fn log_to_slog_level(level: log::Level) -> slog::Level {
        match level {
            log::Level::Trace => slog::Level::Trace,
            log::Level::Debug => slog::Level::Debug,
            log::Level::Info => slog::Level::Info,
            log::Level::Warn => slog::Level::Warning,
            log::Level::Error => slog::Level::Error,
        }
    }
}

impl log::Log for SlogStdLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        use slog::Drain;
        self.0.is_enabled(Self::log_to_slog_level(metadata.level()))
    }

    fn log(&self, r: &log::Record) {

        // log provides Option<&'a str> while slog expects &'static str
        // We can expect log's strings to be static, but we can't safely decide to coerce them
        // into static strings, so we use an interning pool.
        let as_static = |opt: Option<&str>| -> &'static str {
            use intern::Intern;
            match opt {
                None => "<unknown>",
                Some(s) => s.intern()
            }
        };

        let s = slog::RecordStatic {
            location: &slog::RecordLocation {
                file: "<unknown>", // Using 'module' is nicer, so save the interning time.
                line: r.line().unwrap_or(0),
                column: 0,
                function: "<unknown>",
                module: as_static(r.module_path()),
            },
            level: Self::log_to_slog_level(r.metadata().level()),
            tag: r.target(),
        };

        self.0.log(&slog::Record::new(&s, r.args(), slog::BorrowedKV(&())));
    }

    fn flush(&self) {}
}

//-------------------------------------------------------------------------------------------------

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
        let new_log = slog_scope::logger().new(o!("request_id" => req_id));

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
