// @generated automatically by Diesel CLI.

diesel::table! {
    note (id) {
        id -> Integer,
        title -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
        body -> Text,
    }
}
