//Uncomment the following line to see the compiler find 'table!', but fail to find the rest of the macros
//use diesel::*;

table! {
    login_table (user_name) {
        user_name -> Varchar,
        user_psw -> Varchar,
    }
}