use anyhow::{Result, bail};
use crate::{
    http_parsing::*,
    login_table::LoginTable,
};

#[cfg(not(test))] use {
    std::process::exit,
};

#[cfg(test)] use {
    futures::executor::block_on,
};

#[cfg(not(test))]
pub async fn wait_for_shudown() {
    tokio::signal::ctrl_c().await.expect("Failed to initialize Ctrl+C signal handler");
    println!(" Server shutting down...");
    exit(0);
}


pub async fn login(table: &LoginTable, params: &str) -> Result<()> {
    let LoginForm { user_name, password, .. } = parse_login_params(params)?;
    let user_rows = table.query_user(user_name.as_str()).await?;
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
        let signup_query = "uname=ednaldo&psw=pereira&psw-repeat=pereira&remember=on";
        block_on(signup(&mut login_table, signup_query)).unwrap();

        let login_query = "uname=ednaldo&psw=pereira&remember=on";
        block_on(login(&login_table, login_query)).unwrap();
    }
}