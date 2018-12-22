// Since slog also defines log's macros, we can't blindly import "slog::*" but always repeating
// these imports is a pain. So just `use log_macros::*` and you're all set.
pub use log::{debug, error, info, trace, warn};
pub use slog::{slog_crit, slog_debug, slog_error, slog_info, slog_log, slog_o, slog_trace, slog_warn};
