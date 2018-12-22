use serde_derive::Deserialize;

pub mod logs;
pub mod metrics;

use actix_web::http::header;
use actix_web::http::Method;
use actix_web::middleware::cors;
use actix_web::{server, App, AsyncResponder, HttpRequest, HttpResponse, Json, Path, Query, Responder, State};
use std::sync::Once;

use risso_api::context::*;
use risso_api::log_macros::*;

use futures::prelude::*;

use crate::logs::RequestLogger;

mod log_macros {
    // Since slog also defines log's macros, we can't blindly import "slog::*" but always repeating
    // these imports is a pain. So just `use crate::log_macros::*` and you're all set.
    pub use log::{debug, error, info, trace, warn};
    pub use slog::{slog_crit, slog_debug, slog_error, slog_info, slog_log, slog_o, slog_trace, slog_warn};
}

fn unsubscribe(state: State<ApiContext>, path: Path<(String, String, String)>) -> impl Responder {
    let (id, email, key) = path.into_inner();

    risso_api::unsubscribe(&state, id, email, key).map(Json).responder()
}

pub fn view(id: Path<String>, req: HttpRequest<ApiContext>) -> impl Responder {
    let plain = req.query().contains_key("plain");

    format!(
        "id={:?}, match_id={:?}, plain={:?}, matchInfo={:?}",
        id,
        req.match_info().get("id"),
        plain,
        req.match_info()
    )
}

#[derive(Deserialize)]
pub struct NewCommentParams {
    pub uri: String,
}

pub fn new_comment(
    _log: RequestLogger,
    state: State<ApiContext>,
    req: Query<NewCommentParams>,
    body: Json<risso_api::NewComment>,
) -> impl Responder {
    risso_api::new_comment(&state, req.into_inner().uri, body.into_inner())
        .map(Json)
        .responder()
}

pub fn fetch(log: RequestLogger, state: State<ApiContext>, req: Query<risso_api::FetchRequest>) -> impl Responder {
    slog_info!(log, "Fetching comments");

    risso_api::fetch(&state, req.into_inner()).map(Json).responder()
}

//--------------------------------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct ActixConfig {
    listen_addr: String,
    allowed_origins: Vec<String>,
}

pub fn main() -> Result<(), failure::Error> {
    let (_guard, _log) = logs::setup_slog();

    info!("Starting...");

    let config = risso_api::CONFIG.get::<ActixConfig>("actix")?;

    let listen_addr = config.listen_addr;
    let allowed_origins = config.allowed_origins;

    let api_builder = ApiBuilder::new()?;
    let api = api_builder.build();

    let metrics_builder = metrics::MiddlewareBuilder::new()?;

    let srv = server::new(move || {
        App::with_state(api.clone())
            .route("/", Method::GET, fetch)
            .route("/new", Method::POST, new_comment)
            .route("/count", Method::GET, get_counts)
            .route("/counts", Method::POST, post_counts)
            .route("/feed", Method::GET, feed)
            .route("/id/{id}", Method::GET, view)
            .route("/id/{id}/unsubscribe/{email}/{key}", Method::GET, unsubscribe)
            .route(
                "/id/{id}/{action:(edit|delete|activate)}/{key}",
                Method::GET,
                comment_action,
            )
            .route(
                "/id/{id}/{action:(edit|delete|activate)}/{key}",
                Method::POST,
                comment_action,
            )
            .route("/id/{id}/like", Method::POST, like)
            .route("/id/{id}/dislike", Method::POST, dislike)
            .route("/preview", Method::POST, preview)
            .route("/admin", Method::GET, admin)
            .route("/metrics", Method::GET, metrics::handler)
            .middleware(metrics_builder.build())
            .middleware(build_cors(&allowed_origins))
            .middleware(actix_web_requestid::RequestIDHeader)
    });

    srv.bind(listen_addr)?.run();

    Ok(())
}

fn build_cors(origins: &Vec<String>) -> cors::Cors {
    static CHECK: Once = Once::new();
    CHECK.call_once(|| {
        if origins.is_empty() {
            warn!("No CORS origins set. Make sure you configure 'actix.allowed_origins'");
        }
    });

    let mut cors = cors::Cors::build();

    for origin in origins {
        cors.allowed_origin(&origin);
    }

    cors.allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE])
        .max_age(3600);

    cors.finish()
}

//--------------------------------------------------------------------------------------------------
// Stubs

fn todo() -> HttpResponse {
    HttpResponse::NotImplemented().body("Not implemented yet!")
}

fn get_counts(_state: State<ApiContext>) -> HttpResponse {
    todo()
}

fn post_counts(_state: State<ApiContext>) -> HttpResponse {
    todo()
}

fn feed(_state: State<ApiContext>) -> HttpResponse {
    todo()
}

fn comment_action(_state: State<ApiContext>) -> HttpResponse {
    todo()
}

fn like(_state: State<ApiContext>) -> HttpResponse {
    todo()
}

fn dislike(_state: State<ApiContext>) -> HttpResponse {
    todo()
}

fn preview(_state: State<ApiContext>) -> HttpResponse {
    todo()
}

fn admin(_state: State<ApiContext>) -> HttpResponse {
    todo()
}
