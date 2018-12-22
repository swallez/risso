#![allow(proc_macro_derive_resolution_fallback)]

use diesel::*;
// Schema generated with `diesel print-schema` and hand-edited to remove lots of Nullable

table! {
    preferences (key) {
        key -> Text,
        value -> Text,
    }
}

table! {
    threads (id) {
        id -> Integer,
        uri -> Text, // Unique
        title -> Text, // FIXME: check if it has to be nullable
    }
}

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
