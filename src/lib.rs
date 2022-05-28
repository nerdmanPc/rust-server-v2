use std::process::exit;
use tokio::{
    fs::File,
    io::AsyncReadExt,
    io::AsyncWriteExt,
};
use regex::Regex;
use anyhow::{Result, bail};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use tokio_postgres::{Client, Row, NoTls};


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

pub struct LoginTable{
    client: Client,
} 

impl LoginTable {

    pub async fn new() -> Result<Self> {

        let (client, connection) = tokio_postgres::connect("host=localhost user=login_server password='le_server' dbname=login", NoTls).await?;
        let wait_for_connection = async move {
            connection.await.expect("Error setting up database connection!")
        };
        tokio::spawn(wait_for_connection);
        client.execute(include_str!("../database/create_login_table.sql"), &[]).await?;

        Ok(Self{client})
    }

    async fn insert_user(&self, user_name: &str, password: &str) -> Result<()> {
        self.client.execute(include_str!("../database/insert_user.sql"),  &[&user_name, &password]).await?;
        Ok(())
    }

    async fn query_user(&self, user_name: &str) -> Result<Vec<Row>> {
        let rows = self.client.query(include_str!("../database/query_user.sql"), &[&user_name]).await?;
        Ok(rows)
    }

    pub async fn login(&self, params: &str) -> Result<()> {
        let LoginForm { user_name, password, .. } = parse_login_params(params)?;
        let user_rows = self.query_user(user_name.as_str()).await?;
        if user_rows.is_empty() {
            bail!("User {} not found!", user_name);
        }
        let registered_psw: &str = user_rows[0].get(1);
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
}