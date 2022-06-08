#[cfg(not(test))]
use diesel::table;

#[cfg(not(test))]
table! {
    login_table (user_name) {
        user_name -> Varchar,
        user_psw -> Varchar,
    }
}
