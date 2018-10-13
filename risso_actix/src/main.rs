
extern crate diesel;
extern crate actix_web;
extern crate risso_api;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use actix_web::{server, App, HttpRequest, Responder, Json};

use risso_api::models::*;
use risso_api::models::threads::dsl::*;


fn greet(req: &HttpRequest) -> impl Responder {
    let to = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", to)
}


fn get_threads(_req: &HttpRequest) -> impl Responder {
    let database_url = "temp/comments.db";
    let cnx = SqliteConnection::establish(database_url).expect("Can't connect to database");

    let all_threads = threads.load::<Thread>(&cnx).unwrap();

    Json(all_threads)

}


pub fn main() {
    let addr = "127.0.0.1:8000";
    println!("Listening on http://{}", addr);
    server::new(|| {
        App::new()
            .resource("/", |rsrc| rsrc.f(get_threads))
            .resource("/{name}", |rsrc| rsrc.f(greet))
    })
    .bind(addr)
    .expect("Can not bind to port 8000")
    .run();
}
