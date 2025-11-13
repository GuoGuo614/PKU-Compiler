use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use koopa::ir as ir;
use ir::{Value, Function};

#[derive(Clone)]
pub enum Sym {
    Const(i32),
    Var(Value),
    Func(Function),
    Array(Value, Vec<usize>),    
    ArrayPointer(Value, Vec<usize>),
    Pointer(Value)                   
}

pub struct SymbolTable {
    scopes: VecDeque<Rc<HashMap<String, Sym>>>,
}

impl SymbolTable {
    // 用于生成全局作用域
    pub fn new() -> Self {
        let mut table = SymbolTable {
            scopes: VecDeque::new(),
        };
        table.enter_scope();
        table
    }

    // 用于生成函数作用域
    pub fn with_parent(parent: &SymbolTable) -> Self {
        let mut table = SymbolTable {
            scopes: VecDeque::new(),
        };
        if let Some(global_scope) = parent.scopes.front() {
            table.scopes.push_back(Rc::clone(global_scope));
        }
        table.enter_scope();
        table
    }

    pub fn insert_const(&mut self, name: String, value: i32) {
        if self.symbol_exist_current(&name) {
            panic!("Symbol '{}' already exists in current scope!", name);
        }
        Rc::get_mut(self.scopes.back_mut().unwrap())
            .expect("No active scope")
            .insert(name, Sym::Const(value));
    }

    pub fn insert_var(&mut self, name: &str, alloc: Value) {
        if self.symbol_exist_current(name) {
            panic!("Symbol '{}' already exists in current scope!", name);
        }
        Rc::get_mut(self.scopes.back_mut().unwrap())
            .expect("No active scope")
            .insert(name.to_string(), Sym::Var(alloc));
    }

    pub fn insert_array(&mut self, name: &str, alloc: Value, sizes: Vec<usize>) {
        if self.symbol_exist_current(name) {
            panic!("Symbol '{}' already exists in current scope!", name);
        }
        Rc::get_mut(self.scopes.back_mut().unwrap())
            .expect("No active scope")
            .insert(name.to_string(), Sym::Array(alloc, sizes));
    }

    pub fn insert_array_pointer(&mut self, name: &str, ptr: Value, sizes: Vec<usize>) {
        if self.symbol_exist_current(name) {
            panic!("Symbol '{}' already exists in current scope!", name);
        }
        Rc::get_mut(self.scopes.back_mut().unwrap())
            .expect("No active scope")
            .insert(name.to_string(), Sym::ArrayPointer(ptr, sizes));
    }

    pub fn insert_ptr(&mut self, name: &str, alloc: Value) {
        if self.symbol_exist_current(name) {
            panic!("Symbol '{}' already exists in current scope!", name);
        }
        Rc::get_mut(self.scopes.back_mut().unwrap())
            .expect("No active scope")
            .insert(name.to_string(), Sym::Pointer(alloc));
    }

    pub fn insert_func(&mut self, name: &str, func: &Function) {
        if self.symbol_exist_current(name) {
            panic!("Symbol '{}' already exists in current scope!", name);
        }
        Rc::get_mut(self.scopes.back_mut().unwrap())
            .expect("No active scope")
            .insert(name.to_string(), Sym::Func(*func));
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

    pub fn get_all_sym(&self, name: &str) -> Option<&Sym> {
        self.scopes.iter().rev()
            .find_map(|s| s.get(name))
    }

    pub fn get_var(&self, name: &str) -> Option<&Value> {
        match self.get_all_sym(name)? {
            Sym::Var(alloc) => Some(alloc),
            _ => None,
        }
    }

    pub fn get_const(&self, name: &str) -> Option<&i32> {
        match self.get_all_sym(name)? {
            Sym::Const(num) => Some(num),
            _ => None,
        }
    }

    pub fn get_func(&self, name: &str) -> Option<&Function> {
        match self.get_all_sym(name)? {
            Sym::Func(func) => Some(func),
            _ => None,
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push_back(Rc::new(HashMap::new()));
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop_back();
    }
}