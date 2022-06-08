use regex::Regex;
use anyhow::{Result, bail};

#[cfg(not(test))] use {
    std::{
        process::exit,
        env,
    },
    diesel::{
        prelude::*,
        pg::PgConnection,
    },
    dotenv::dotenv,
};

#[cfg(test)] use {
    std::collections::HashMap,
    futures::executor::block_on,
};

pub mod schema;
pub mod models;

#[cfg(not(test))]
pub async fn wait_for_shudown() {
    tokio::signal::ctrl_c().await.expect("Failed to initialize Ctrl+C signal handler");
    println!(" Server shutting down...");
    exit(0);
}

#[derive(Debug, Eq, PartialEq)]
pub struct LoginForm {
    pub user_name: String,
    pub password: String,
    pub remember: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SignupForm {
    pub user_name: String,
    pub password: String,
    pub psw_repeat: String,
    pub remember: bool,
}

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
    async fn insert_user(&self, user_name: &str, password: &str) -> Result<()> {
        //self.client.execute(include_str!("../database/insert_user.sql"),  &[&user_name, &password]).await?;
        Ok(())
    }

    #[cfg(test)]
    async fn insert_user(&mut self, user_name: &str, password: &str) -> Result<()> {
        self.table.insert(user_name.to_string(), password.to_string());
        Ok(())
    }

    #[cfg(not(test))]
    async fn query_user(&self, usr_name: &str) -> Result<Vec<models::User>> {

        use schema::login_table::dsl::*;

        let results = login_table.filter(user_name.eq(usr_name)).load::<models::User>(&self.connection)?;
        Ok(results)
    }

    #[cfg(test)]
    async fn query_user(&self, user_name: &str) -> Result<Vec<models::User>> {
        let rows: Vec<models::User> = self.table.iter().filter( |pair| {
            let (usr, _psw) = pair;
            usr.as_str() == user_name
        }).map( |pair| {
            let (name, psw) = pair;
            models::User {name: name.clone(), psw: psw.clone()}
        }).collect();
        Ok(rows)
    }

    pub async fn login(&self, params: &str) -> Result<()> {
        let LoginForm { user_name, password, .. } = parse_login_params(params)?;
        let user_rows = self.query_user(user_name.as_str()).await?;
        if user_rows.is_empty() {
            bail!("User {} not found!", user_name);
        }
        #[cfg(not(test))]
        let registered_psw: &str = user_rows[0].psw.as_str();
        #[cfg(test)]
        let registered_psw: &str = user_rows[0].psw.as_str();

        if registered_psw != password.as_str() {
            bail!("Wrong password!");
        }
        Ok(())
    }

    pub async fn signup(&mut self, params: &str) -> Result<()> {
        let SignupForm { user_name, password, psw_repeat, .. } = parse_signup_params(params)?;
        let user_rows = self.query_user(user_name.as_str()).await?;
        if !user_rows.is_empty() {
            bail!("User {} already exists!", user_name)
        }
        if password != psw_repeat {
            bail!("Passwords do not match")
        }
        self.insert_user(user_name.as_str(), password.as_str()).await?;
        Ok(())
    }
}

#[cfg(not(test))]
fn connect_to_database() -> Result<PgConnection> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")?;
    let connection = PgConnection::establish(database_url.as_str())?;
    Ok(connection)
}

fn parse_login_params(query: &str) -> Result<LoginForm> {
    let login_regex = Regex::new(r"^uname=([[:alpha:]]*)&psw=(\w*)&remember=(on|off)$")?;
    let regex_capture = login_regex.captures(query);
    if regex_capture.is_none() {
        bail!("Malformed login query: {}", query)
    }
    let regex_capture = regex_capture.unwrap();
    let user_name = regex_capture.get(1).unwrap().as_str().to_owned();
    let password = regex_capture.get(2).unwrap().as_str().to_owned();
    let remember =  match regex_capture.get(3).unwrap().as_str() {
        "on" => { true },
        "off" => { false },
        _ => { panic!("Failed to compile login regex!") }
    };

    Ok( LoginForm {
        user_name,
        password,
        remember,
    })
}

fn parse_signup_params(query: &str) -> Result<SignupForm>{
    let signup_regex = Regex::new(r"^uname=([[:alpha:]]*)&psw=(\w*)&psw-repeat=(\w*)&remember=(on|off)$")?;
    let regex_capture = signup_regex.captures(query);
    if regex_capture.is_none() {
        bail!("Malformed signup query: {}", query)
    }
    let regex_capture = regex_capture.unwrap();
    let user_name = regex_capture.get(1).unwrap().as_str().to_owned();
    let password = regex_capture.get(2).unwrap().as_str().to_owned();
    let psw_repeat = regex_capture.get(3).unwrap().as_str().to_owned();
    let remember =  match regex_capture.get(4).unwrap().as_str() {
        "on" => { true },
        "off" => { false },
        _ => { panic!("Failed to compile signup regex!") }
    };

    Ok( SignupForm {
        user_name,
        password,
        psw_repeat,
        remember,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_login_parse() {
        let correct_query = "uname=ednaldo&psw=pereira&remember=on";
        let result = parse_login_params(correct_query).unwrap();
        let expected = LoginForm {
            user_name: "ednaldo".to_owned(),
            password: "pereira".to_owned(),
            remember: true,
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn correct_signup_parse() {
        let correct_query = "uname=ednaldo&psw=pereira&psw-repeat=pereira&remember=on";
        let result = parse_signup_params(correct_query).unwrap();
        let expected = SignupForm {
            user_name: "ednaldo".to_owned(),
            password: "pereira".to_owned(),
            psw_repeat: "pereira".to_owned(),
            remember: true,
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn login_after_signup() {
        let mut login_table = LoginTable::new();
        let signup_query = "uname=ednaldo&psw=pereira&psw-repeat=pereira&remember=on";
        block_on(login_table.signup(signup_query)).unwrap();

        let login_query = "uname=ednaldo&psw=pereira&remember=on";
        block_on(login_table.login(login_query)).unwrap();
    }
}