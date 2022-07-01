table! {
    EmployeeInfo (id) {
        id -> Unsigned<Bigint>,
        firstname -> Varchar,
        lastname -> Varchar,
    }
}

table! {
    EmployeeLogins (id) {
        id -> Unsigned<Bigint>,
        info_id -> Unsigned<Bigint>,
        username -> Varchar,
        hash -> Varchar,
    }
}

table! {
    EmployeeSessions (id) {
        id -> Unsigned<Bigint>,
        e_id -> Unsigned<Bigint>,
        token -> Varchar,
        expires -> Datetime,
    }
}

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

joinable!(EmployeeLogins -> EmployeeInfo (info_id));
joinable!(EmployeeSessions -> EmployeeLogins (e_id));
joinable!(Sessions -> Users (user_id));

allow_tables_to_appear_in_same_query!(
    EmployeeInfo,
    EmployeeLogins,
    EmployeeSessions,
    PendingUsers,
    Sessions,
    Users,
);
