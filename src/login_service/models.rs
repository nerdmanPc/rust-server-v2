use crate::schema::login_table;

#[derive(Queryable)]
pub struct User {
    pub name: String,
    pub psw: String,
}

#[derive(Insertable)]
#[table_name="login_table"]
pub struct NewUser<'a> {
    pub user_name: &'a str,
    pub user_psw: &'a str,
}