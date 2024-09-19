// @generated automatically by Diesel CLI.

diesel::table! {
    note (id) {
        id -> Integer,
        title -> Text,
        body -> Text,
        created_at -> Timestamp,
    }
}
