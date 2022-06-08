use {
    crate::models::User,
    anyhow::Result
};

#[cfg(not(test))] use {
    diesel::pg::PgConnection,
    diesel::prelude::*,
    dotenv::dotenv,
    std::env,
};

#[cfg(test)] use {
    std::collections::HashMap,
};

pub struct LoginTable {
    #[cfg(not(test))]
    connection: PgConnection,
    #[cfg(test)]
    table: HashMap<String, String>,
}

impl LoginTable {

    #[cfg(not(test))]
    pub async fn new() -> Result<Self> {
        let connection = connect_to_database()?;
        Ok( Self{connection} )
    }

    #[cfg(test)]
    pub fn new() -> Self {
        let table = HashMap::new();
        Self { table }
    }

    #[cfg(not(test))]
    pub async fn insert_user(&self, user_name: &str, password: &str) -> Result<()> {
        //self.client.execute(include_str!("../database/insert_user.sql"),  &[&user_name, &password]).await?;
        Ok(())
    }

    #[cfg(test)]
    pub async fn insert_user(&mut self, user_name: &str, password: &str) -> Result<()> {
        self.table.insert(user_name.to_string(), password.to_string());
        Ok(())
    }

    #[cfg(not(test))]
    pub async fn query_user(&self, name: &str) -> Result<Vec<User>> {
        use crate::schema::login_table::dsl::*;
        let results = login_table.filter(user_name.eq(name)).load::<User>(&self.connection)?;
        Ok(results)
    }

    #[cfg(test)]
    pub async fn query_user(&self, user_name: &str) -> Result<Vec<User>> {
        let rows: Vec<User> = self.table.iter().filter( |pair| {
            let (usr, _psw) = pair;
            usr.as_str() == user_name
        }).map( |pair| {
            let (name, psw) = pair;
            User {name: name.clone(), psw: psw.clone()}
        }).collect();
        Ok(rows)
    }
}

#[cfg(not(test))]
pub fn connect_to_database() -> Result<PgConnection> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")?;
    let connection = PgConnection::establish(database_url.as_str())?;
    Ok(connection)
}