pub use {
    anyhow::Result,
};

#[derive(Debug, Eq, PartialEq)]
pub struct WebToken {
    
}

impl WebToken {
    pub fn new(user_name: &str) -> Result<Self> {
        return Ok(WebToken{})
    }
}

struct Claims {
    sub: String,
    user_name: String,
    exp: u64,
}