#![allow(dead_code)]

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate serde;

#[macro_use]
extern crate serde_derive;

extern crate serde_json;
extern crate chrono;

extern crate failure;


pub mod api;
pub mod models;

// newtype: use defer to pull wrapped type's methods
// https://doc.rust-lang.org/book/second-edition/ch19-03-advanced-traits.html#using-the-newtype-pattern-to-implement-external-traits-on-external-types

pub type ThreadId = i32;
pub type CommentId = i32;

enum CommentMode {
    Valid, // 1
    Pending, // 2
    SoftDeleted, // 4
}
