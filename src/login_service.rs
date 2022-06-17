use anyhow::{Result, bail};

#[cfg(not(test))] use {
    std::process::exit,
};

#[cfg(test)] use {
    futures::executor::block_on,
};

mod http_parsing; use http_parsing::*;
pub mod login_table; pub use login_table::LoginTable;
pub mod sessions_table; pub use sessions_table::SessionsTable;
pub mod models; 

#[cfg(not(test))]
pub async fn wait_for_shudown() {
    tokio::signal::ctrl_c().await.expect("Failed to initialize Ctrl+C signal handler");
    println!("\nServer shutting down...");
    exit(0);
}


pub async fn login(login_table: &LoginTable, sessions_table: &mut SessionsTable, params: &str) -> Result<u64> {
    let LoginForm { user_name, password, .. } = parse_login_params(params)?;
    let user_rows = login_table.query_user(user_name.as_str()).await?;
    if user_rows.is_empty() {
        bail!("User {} not found!", user_name);
    }
    let registered_psw: &str = user_rows[0].psw.as_str();
    if registered_psw != password.as_str() {
        bail!("Wrong password!");
    }
    let session_cookie = sessions_table.new_session(user_name.as_str())?;
    Ok(session_cookie)
}

pub async fn signup(table: &mut LoginTable, params: &str) -> Result<()> {
    let SignupForm { user_name, password, psw_repeat, .. } = parse_signup_params(params)?;
    let user_rows = table.query_user(user_name.as_str()).await?;
    if !user_rows.is_empty() {
        bail!("User {} already exists!", user_name)
    }
    if password != psw_repeat {
        bail!("Passwords do not match")
    }
    table.insert_user(user_name.as_str(), password.as_str()).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_after_signup() {
        let mut login_table = LoginTable::new();
        let mut sessions_table = SessionsTable::new().unwrap();
        let signup_query = "uname=ednaldo&psw=pereira&psw-repeat=pereira&remember=on";

        block_on(signup(&mut login_table, signup_query)).unwrap();

        let login_query = "uname=ednaldo&psw=pereira&remember=on";

        block_on(login(&login_table, &mut sessions_table, login_query)).unwrap();
    }

    #[test]
    fn login_absent_user() {
        let login_table = LoginTable::new();
        let mut sessions_table = SessionsTable::new().unwrap();
        let login_query = "uname=ednaldo&psw=pereira&remember=on";

        let login_result = block_on(login(&login_table, &mut sessions_table, login_query));

        assert!(login_result.is_err());
    }

    #[test]
    fn login_with_wrong_password() {
        let mut login_table = LoginTable::new();
        let mut sessions_table = SessionsTable::new().unwrap();
        let signup_query = "uname=ednaldo&psw=pereira&psw-repeat=pereira&remember=on";
        block_on(signup(&mut login_table, signup_query)).unwrap();
        let login_query = "uname=ednaldo&psw=fleig&remember=on";

        let login_result = block_on(login(&login_table, &mut sessions_table, login_query));

        assert!(login_result.is_err());
    }

    #[test]
    fn login_twice() {
        let mut login_table = LoginTable::new();
        let mut sessions_table = SessionsTable::new().unwrap();
        let signup_query = "uname=ednaldo&psw=pereira&psw-repeat=pereira&remember=on";
        block_on(signup(&mut login_table, signup_query)).unwrap();
        let login_query = "uname=ednaldo&psw=pereira&remember=on";

        let first_login_result = block_on(login(&login_table, &mut sessions_table, login_query)).unwrap();
        let second_login_result = block_on(login(&login_table, &mut sessions_table, login_query)).unwrap();

        assert_ne!(first_login_result, second_login_result);
    }

    #[test]
    fn signup_existing_user() {
        let mut login_table = LoginTable::new();
        let signup_query = "uname=ednaldo&psw=pereira&psw-repeat=pereira&remember=on";
        block_on(signup(&mut login_table, signup_query)).unwrap();

        let second_signup_result = block_on(signup(&mut login_table, signup_query));

        assert!(second_signup_result.is_err());
    }

    #[test]
    fn signup_with_different_passwords() {
        let mut login_table = LoginTable::new();
        let signup_query = "uname=ednaldo&psw=pereira&psw-repeat=fleig&remember=on";
        
        let signup_result = block_on(signup(&mut login_table, signup_query));

        assert!(signup_result.is_err());
    }
}