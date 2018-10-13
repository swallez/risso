#[macro_use] extern crate tower_web;
extern crate diesel;
extern crate failure;
extern crate risso_api;
extern crate serde_json;
extern crate serde;

use tower_web::response::SerdeResponse;
use tower_web::response::Response;
use tower_web::ServiceBuilder;

use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::sqlite::SqliteConnection;

use failure::Error;

type DBPool = Pool<ConnectionManager<SqliteConnection>>;

use risso_api::models::*;
use risso_api::models::threads::dsl::*;

#[derive(Response)]
struct MyResponse(Vec<Thread>);

struct RissoAPI{
    pool: DBPool
}

impl RissoAPI {
    fn hello_world(&self) -> Result<MyResponse, Error> {
        let cnx = self.pool.get()?;
        let all_threads: Vec<Thread> = threads.load::<Thread>(&cnx)?;

        Ok(MyResponse(all_threads))
    }
}

impl_web! {
    impl RissoAPI {
        #[get("/")]
        #[content_type("application/json")]
        fn hello_world0(&self) -> Result<MyResponse, Error> {
            self.hello_world()
        }
    }
}


pub fn main() -> Result<(), Error> {

    let database_url = "temp/comments.db2";

    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = Pool::builder().build(manager).expect("Failed to create pool.");

    let addr = "127.0.0.1:8080".parse().expect("Invalid address");
    println!("Listening on http://{}", addr);

    let _ = ServiceBuilder::new()
        .resource(RissoAPI{pool: pool})
        .run(&addr)?;

    Ok(())
}

// TODO:
// - CORS
// - access log
// - error log
