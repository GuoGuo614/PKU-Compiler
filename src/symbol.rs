use std::collections::HashMap;

pub struct SymbolTable {
    map: HashMap<String, i32>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            map: HashMap::new(),
        }
    }

    pub fn insert_symbol(&mut self, name: String, value: i32) {
        self.map.insert(name, value);
    }

    pub fn symbol_exist(&self, name: &str) -> bool {
        self.map.contains_key(name)
    }

    pub fn get_symbol_value(&self, name: &str) -> Option<&i32> {
        self.map.get(name)
    }
}