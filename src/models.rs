use diesel::Queryable;

#[derive(Queryable)]
pub struct User {
    pub name: String,
    pub psw: String,
}