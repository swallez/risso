#![allow(proc_macro_derive_resolution_fallback)]

mod dieselext;

use self::dieselext::*;

// `table!` macros fenerated with `cargo print-schema` and hand-edited to remove lots of Nullable

// CREATE TABLE preferences (key VARCHAR PRIMARY KEY, value VARCHAR );

table! {
    preferences (key) {
        key -> Text,
        value -> Text,
    }
}

#[derive(Queryable, Debug)]
pub struct Preference {
    pub key: String,
    pub value: String,
}

//----------

// CREATE TABLE threads (id INTEGER PRIMARY KEY, uri VARCHAR(256) UNIQUE, title VARCHAR(256));

table! {
    threads (id) {
        id -> Integer,
        uri -> Text, // Unique
        title -> Text, // FIXME: check if it has to be nullable
    }
}

#[derive(Queryable, Debug, Serialize, Deserialize)]
pub struct Thread {
    pub id: i32,
    pub uri: String,
    pub title: String,
}

//----------

// CREATE TABLE comments (
//   tid REFERENCES threads(id), id INTEGER PRIMARY KEY, parent INTEGER,
//   created FLOAT NOT NULL, modified FLOAT, mode INTEGER, remote_addr VARCHAR,
//   text VARCHAR, author VARCHAR, email VARCHAR, website VARCHAR,
//   likes INTEGER DEFAULT 0, dislikes INTEGER DEFAULT 0, voters BLOB NOT NULL
// );
//
// ALTER TABLE comments ADD COLUMN notification INTEGER NOT NULL DEFAULT 0;
//
// CREATE TRIGGER remove_stale_threads AFTER DELETE ON comments BEGIN
//   DELETE FROM threads WHERE id NOT IN (SELECT tid FROM comments);
// END;

table! {
    comments (id) {
        #[sql_name = "tid"]
        thread_id -> Integer,
        id -> Integer,
        parent -> Nullable<Integer>,
        created -> Double, // print_schema generates a Float
        modified -> Nullable<Double>,
        mode -> Integer, // status: 1 = valid, 2 = pending, # 4 = soft-deleted (cannot hard delete because of replies)
        remote_addr -> Text,
        text -> Text,
        author -> Nullable<Text>,
        email -> Nullable<Text>,
        website -> Nullable<Text>,
        likes -> Integer,
        dislikes -> Integer,
        notification -> Bool,
        voters -> Binary, // bloom_filter(remote_addr), initialized with poster's address so he can't vote on himself
    }
}

joinable!(comments -> threads (thread_id));
allow_tables_to_appear_in_same_query!(comments, threads);

#[derive(Queryable, Debug)]
pub struct Comment {
    pub thread_id: i32,
    pub id: i32,
    pub parent: Option<i32>,
    pub created: FloatDateTime,
    pub modified: Option<FloatDateTime>,
    pub mode: i32,
    pub remote_addr: String,
    pub text: String,
    pub author: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub likes: i32,
    pub dislikes: i32,
    pub notification: bool,
    pub voters: Vec<u8>,
}



