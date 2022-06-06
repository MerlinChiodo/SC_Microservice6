table! {
    PendingUsers (id) {
        id -> Unsigned<Bigint>,
        citizen -> Bigint,
        code -> Varchar,
    }
}

table! {
    Sessions (id) {
        id -> Unsigned<Bigint>,
        user_id -> Unsigned<Bigint>,
        token -> Varchar,
        expires -> Datetime,
    }
}

table! {
    Users (id) {
        id -> Unsigned<Bigint>,
        username -> Varchar,
        hash -> Varchar,
    }
}

joinable!(Sessions -> Users (user_id));

allow_tables_to_appear_in_same_query!(
    PendingUsers,
    Sessions,
    Users,
);
