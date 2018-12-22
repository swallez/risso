#![allow(proc_macro_derive_resolution_fallback)]

use crate::context;
use crate::dieselext;
use crate::dieselext::*;
use crate::log_macros::*;
use crate::schema::*;

use diesel::prelude::*;
use diesel::result::QueryResult;
use diesel::sql_types::Bool;

use serde_derive::{Deserialize, Serialize};

use prometheus::Counter;

/// isso [handles comment mode values in a weird way][1]: each possible mode is a bit, and selection
/// happens by testing against a bitmask rather than using SQL 'IN'.
///
/// Value 5 is used by default, selecting valid and soft-deleted comments.
///
/// [1]: https://github.com/posativ/isso/blob/f2333d716d661a5ab1d0102b3f5890080267755a/isso/db/comments.py#L182

pub enum CommentMode {
    Valid = 1,
    Pending = 2,
    SoftDeleted = 4, // cannot hard delete because of replies
}

impl CommentMode {
    pub fn mask(opt_mode: Option<i32>) -> diesel::expression::SqlLiteral<Bool> {
        let mode = opt_mode.unwrap_or(5);
        diesel::dsl::sql::<diesel::sql_types::Bool>(&format!("({} | comments.mode) = {}", mode, mode))
    }
}

#[derive(Queryable, Debug)]
pub struct Preference {
    pub key: String,
    pub value: String,
}

#[derive(Clone, Queryable, Debug, Serialize, Deserialize)]
pub struct Thread {
    pub id: i32,
    pub uri: String,
    pub title: String,
}

#[derive(Queryable, Debug, Serialize)]
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

lazy_static! {
    static ref HTTP_COUNTER: Counter = register_counter!(opts!(
        "example_http_requests_total",
        "Total number of HTTP requests made.",
        labels! {"handler" => "all",}
    ))
    .unwrap();
}

impl Comment {
    /// Return comments for `uri` with `mode`.
    #[allow(clippy::too_many_arguments)]
    pub fn fetch(
        cnx: &context::Connection,
        uri: String,
        mode: Option<i32>,
        after: f64,
        parent: Option<i32>,
        order_by: Option<String>,
        asc: bool,
        limit: Option<i64>,
    ) -> QueryResult<Vec<Self>> {
        use crate::schema::comments;
        use crate::schema::threads;

        let mut q = comments::table
            .inner_join(threads::table)
            .select(comments::all_columns)
            .filter(
                threads::uri
                    .eq(uri)
                    .and(CommentMode::mask(mode))
                    .and(comments::created.gt(after)),
            )
            .into_boxed();

        q = match parent {
            None => q.filter(comments::parent.is_null()),
            Some(0) => q, // 'any' in the python version
            Some(id) => q.filter(comments::parent.eq(id)),
        };

        q = if asc {
            // https://stackoverflow.com/a/48034647
            match &order_by.unwrap_or_else(|| String::from("id"))[..] {
                "created" => q.order(comments::created.asc()),
                "modified" => q.order(comments::modified.asc()),
                "likes" => q.order(comments::likes.asc()),
                "dislikes" => q.order(comments::dislikes.asc()),
                _ => q.order(comments::id.asc()),
            }
        } else {
            // FIXME: find a way to avoid this repetition...
            match &order_by.unwrap_or_else(|| String::from("id"))[..] {
                "created" => q.order(comments::created.desc()),
                "modified" => q.order(comments::modified.desc()),
                "likes" => q.order(comments::likes.desc()),
                "dislikes" => q.order(comments::dislikes.desc()),
                _ => q.order(comments::id.desc()),
            }
        };

        q = match limit {
            None => q,
            Some(limit) => q.limit(limit),
        };

        trace!("{:?}", diesel::debug_query::<context::DB, _>(&q));

        q.load(cnx)
    }

    /// Return comment count for main thread and all reply threads for one url.
    pub fn reply_count(
        cnx: &context::Connection,
        uri: String,
        mode: Option<i32>,
        after: f64,
    ) -> QueryResult<Vec<(Option<i32>, i64)>> {
        let stmt = comments::table
            .inner_join(threads::table)
            .select((comments::parent, dieselext::count_star()))
            .filter(
                threads::uri
                    .eq(uri)
                    .and(CommentMode::mask(mode))
                    .and(comments::created.gt(after)),
            )
            .group_by(comments::parent);

        trace!("{:?}", diesel::debug_query::<context::DB, _>(&stmt));

        stmt.load(cnx)
    }
}
