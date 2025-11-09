use std::collections::{HashMap, VecDeque};
use koopa::ir as ir;
use ir::Value;

pub enum Sym {
    Const(i32),
    Var(Value),
}

pub struct SymbolTable {
    scopes: VecDeque<HashMap<String, Sym>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut table = SymbolTable {
            scopes: VecDeque::new(),
        };
        table.enter_scope();
        table
    }

    pub fn insert_const(&mut self, name: String, value: i32) {
        if self.symbol_exist_current(&name) {
            panic!("Symbol '{}' already exists in current scope!", name);
        }
        self.scopes.back_mut()
            .expect("No active scope")
            .insert(name, Sym::Const(value));
    }

    pub fn insert_var(&mut self, name: &str, alloc: Value) {
        if self.symbol_exist_current(name) {
            panic!("Symbol '{}' already exists in current scope!", name);
        }
        self.scopes.back_mut()
            .expect("No active scope")
            .insert(name.to_string(), Sym::Var(alloc));
    }

    // 检查当前作用域是否存在
    pub fn symbol_exist_current(&self, name: &str) -> bool {
        self.scopes.back()
            .map(|s| s.contains_key(name))
            .unwrap_or(false)
    }

    // 检查当前至所有外层作用域是否存在
    pub fn symbol_exist(&self, name: &str) -> bool {
        self.scopes.iter().rev().any(|s| s.contains_key(name))
    }

    pub fn get_const_var(&self, name: &str) -> Option<&Sym> {
        self.scopes.iter().rev()
            .find_map(|s| s.get(name))
    }

    pub fn get_var(&self, name: &str) -> Option<&Value> {
        match self.get_const_var(name)? {
            Sym::Var(alloc) => Some(alloc),
            _ => None,
        }
    }

    pub fn get_const(&self, name: &str) -> Option<&i32> {
        match self.get_const_var(name)? {
            Sym::Const(num) => Some(num),
            _ => None,
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push_back(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop_back();
    }
}