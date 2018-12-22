//! Integration of the prometheus crate with Actix-Web. It provides a middleware to measure
//! request processing time, and a http endpoint to publish the metrics.

use actix_web::middleware::*;
use actix_web::*;

use prometheus::*;

use std::cell::Cell;

/// Actix-Web endpoint to dump prometheus metrics
///
pub fn handler<S>(_req: HttpRequest<S>) -> actix_web::Result<HttpResponse, failure::Error> {
    use prometheus::Encoder;

    let encoder = prometheus::TextEncoder::new();
    let metric_families = prometheus::gather();

    let mut buffer = vec![];
    encoder
        .encode(&metric_families, &mut buffer)
        .map(|_| HttpResponse::Ok().content_type("text/plain").body(buffer))
        .map_err(|err| err.into())
}

#[derive(Clone)]
pub struct MiddlewareBuilder {
    histogram: HistogramVec,
}

impl MiddlewareBuilder {
    /// Creates the metrics builder by registering a new histogram for request metrics.
    /// Actual middlewares must be created using `build()`.
    pub fn builder() -> std::result::Result<Self, failure::Error> {
        let histogram_opts = HistogramOpts::new("req_time", "histo_help").subsystem("actix_web");

        let histogram = HistogramVec::new(histogram_opts, &["status"]).unwrap();

        register(Box::new(histogram.clone()))?;

        Ok(Self { histogram })
    }

    pub fn build(&self) -> MetricsMiddleware {
        MetricsMiddleware {
            histogram: self.histogram.clone(),
            instant: Cell::new(None),
        }
    }
}

use std::time::{Duration, Instant};

/// Actix-Web metrics middleware. It collects response times for http requests, grouped by status
/// code.
///
/// It must be created using a `Builder` or `clone()`'d from another middleware, so that all
/// instances share the same underlying histogram.
///
#[allow(clippy::stutter)]
#[derive(Clone)]
pub struct MetricsMiddleware {
    histogram: HistogramVec,
    instant: Cell<Option<Instant>>,
}

#[inline]
#[allow(clippy::cast_precision_loss)]
fn duration_to_seconds(d: Duration) -> f64 {
    let nanos = f64::from(d.subsec_nanos()) / 1e9;
    d.as_secs() as f64 + nanos
}

impl<S> Middleware<S> for MetricsMiddleware {
    /// Called when request is ready.
    fn start(&self, _req: &HttpRequest<S>) -> actix_web::Result<Started> {
        self.instant.set(Some(Instant::now()));

        Ok(Started::Done)
    }

    /// Called after body stream get sent to peer.
    fn finish(&self, _req: &HttpRequest<S>, resp: &HttpResponse) -> Finished {
        if let Some(start) = self.instant.get() {
            let secs = duration_to_seconds(start.elapsed());

            self.histogram
                .with_label_values(&[resp.status().as_str()])
                .observe(secs);

            self.instant.set(None);
        }

        Finished::Done
    }
}
