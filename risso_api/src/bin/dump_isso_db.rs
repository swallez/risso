extern crate diesel;
extern crate risso_api;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use risso_api::models::*;
//use crate::models::*;

fn main() {
    let database_url = "temp/comments.db";

    let cnx = SqliteConnection::establish(database_url).expect("Can't connect to database");

    use comments::dsl::*;
    let all_comments = comments.load::<Comment>(&cnx).expect("comments");
    for comment in all_comments {
        println!("{:?}", comment);
    }

    use threads::dsl::*;
    let all_threads = threads.load::<Thread>(&cnx).expect("threads");
    for thread in all_threads {
        println!("{:?}", thread);
    }
}
