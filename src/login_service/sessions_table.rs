use {
    anyhow::Result,
    std::collections::HashMap,
};

pub struct SessionsTable {
    table: HashMap<u64, String>,
}

impl SessionsTable {
    pub fn new() -> Result<Self> {
        let table = HashMap::new();
        Ok( Self{table} )
    }
}