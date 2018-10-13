
extern crate chrono;

use chrono::prelude::*;
use super::*;

#[derive(Serialize, Deserialize)]
pub struct FetchRequest {
    /// The URI of the thread to gets comments from.
    uri: String,
    parent: Option<CommentId>,
    limit: Option<usize>,
    nested_limit: Option<usize>,
    after: Option<DateTime<Utc>>,
    plain: bool,
}

pub struct FetchResponse {
    total_replies: usize,
    replies: Vec<CommentResponse>,

}

pub struct CommentResponse {
    id: CommentId,
    mode: CommentMode,
    hash: i32,
    author: Option<String>,
    website: Option<String>,
    created: Option<DateTime<Utc>>,
    modified: Option<DateTime<Utc>>,
    text: String,
    total_replies: usize,
    hidden_replies: usize,
    likes: i32,
    dislikes: i32,
    replies: Vec<CommentResponse>,
}
