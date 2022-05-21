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
use tokio_postgres::{Client, NoTls, Error};


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

#[derive(Serialize, Deserialize)]
pub struct LoginTable {
    table: HashMap<String, String>,
} impl LoginTable {
    
    pub fn new() -> Self {
        let table = HashMap::new();
        Self {
            table,
        }
    }

    pub async fn load_or_create(file_path: &str) -> Result<Self> {

        let (client, connection) = tokio_postgres::connect("host=localhost user=login_server", NoTls).await?;
        let wait_for_connection = async move {
            connection.await.expect("Error setting up database connection!")
        };
        tokio::spawn(wait_for_connection);
        client.execute(include_str!("../database/create_login_table.sql"), &[]).await?;

        let mut open_result = File::open(file_path).await;
        if let Err(_) = open_result {
            open_result = File::create(file_path).await;
            if let Err(e) = open_result {
                bail!("Error initializing login table: {}", e);
            }
            let login_table = Self::new();
            let mut file = open_result.unwrap();
            let file_contents = serde_json::to_vec(&login_table)?;
            file.write(&(*file_contents)).await?;
            return Ok(login_table);
        } else {
            let mut file = open_result.unwrap();
            let mut file_contents = Vec::<u8>::with_capacity(1024);
            file.read_to_end(&mut file_contents).await?;
            let login_table: LoginTable = serde_json::from_slice(&(*file_contents))?;
            return Ok(login_table);
        }
    }

    fn add_user(&mut self, user_name: String, password: String) {
        self.table.insert(user_name, password);
    }

    pub fn login(&self, params: &str) -> Result<()> {
        let LoginForm { user_name, password, .. } = parse_login_params(params)?;
        let registered_psw = self.table.get(user_name.as_str());
        if registered_psw.is_none() {
            bail!("User {} not found!", user_name);
        }
        if registered_psw.unwrap() != password.as_str() {
            bail!("Wrong password!");
        }
        Ok(())
    }

    pub fn signup(&mut self, params: &str) -> Result<()> {
        let SignupForm { user_name, password, psw_repeat, .. } = parse_signup_params(params)?;
        if self.table.contains_key(user_name.as_str()) {
            bail!("User {} already exists!", user_name)
        }
        if password != psw_repeat {
            bail!("Passwords do not match")
        }
        self.add_user(user_name, password);
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

    mod login_table {
        use super::super::*;        
        #[test]
        fn successful_login() {
            let mut login_db = LoginTable::new();
            login_db.add_user("ednaldo".to_owned(), "pereira".to_owned());
            let result = login_db.login("uname=ednaldo&psw=pereira&remember=on");
            assert!(result.is_ok());
        }
        #[test]
        fn successful_signup() {
            let mut login_db = LoginTable::new();
            login_db.signup("uname=ednaldo&psw=pereira&psw-repeat=pereira&remember=on").unwrap();
            let result = login_db.login("uname=ednaldo&psw=pereira&remember=on");
            assert!(result.is_ok());
        }
    }
}