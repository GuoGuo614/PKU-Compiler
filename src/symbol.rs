use std::collections::HashMap;
use koopa::ir as ir;
use ir::Value;

pub enum Sym {
    Const(i32),
    Var(Value),
}

pub struct SymbolTable {
    map: HashMap<String, Sym>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            map: HashMap::new(),
        }
    }

    pub fn insert_const(&mut self, name: String, value: i32) {
        if self.symbol_exist(&name) {
            panic!("Symbol already exists!")
        }
        self.map.insert(name, Sym::Const(value));
    }

    pub fn insert_var(&mut self, name: &str, alloc: Value) {
        if self.symbol_exist(name) {
            panic!("Symbol already exists!")
        }
        self.map.insert(name.to_string(), Sym::Var(alloc));
    }

    pub fn symbol_exist(&self, name: &str) -> bool {
        self.map.contains_key(name)
    }

    pub fn get_const_var(&self, name: &str) -> Option<&Sym> {
        self.map.get(name)
    }

    pub fn get_var(&self, name: &str) -> Option<&Value> {
        if let Sym::Var(alloc) = self.map.get(name).unwrap() {
            Some(alloc)
        } else {
            None
        }
    }

    pub fn get_const(&self, name: &str) -> Option<&i32> {
        if let Sym::Const(num) = self.map.get(name).unwrap() {
            Some(num)
        } else {
            None
        }
    }
}