use std::collections::HashMap;
use koopa::ir;

static AVAILABLE_REGS: [&str; 7] = ["t0", "t1", "t2", "t3", "t4", "t5", "t6"];
pub struct RegAlloc {
    free: Vec<&'static str>,
    reg_map: HashMap<ir::Value, &'static str>,
}

impl RegAlloc {
    pub fn new() -> Self {
        let mut free = AVAILABLE_REGS.to_vec();
        free.reverse();
        Self { 
            free, 
            reg_map: HashMap::new(),
        }
    }

    pub fn alloc(&mut self) -> &'static str {
        self.free.pop().expect("No Free Registers left")
    }

    pub fn release(&mut self, reg: &'static str) {
        self.free.push(reg);
        // 保证一直按顺序分配，但不是那么必要
        // self.free.sort_unstable();
        // self.free.reverse();
    }

    pub fn bind(&mut self, v: ir::Value, reg: &'static str) {
        self.reg_map.insert(v, reg);
    }

    pub fn reg_of(&self, v: ir::Value) -> Option<&'static str> {
        self.reg_map.get(&v).copied()
    }

    pub fn dec_use(&mut self, v: ir::Value) {
        if let Some(reg) = self.reg_map.remove(&v) {
            self.release(reg);
        }
    }
}

pub struct StackFrame {
    pub total: usize,
    pub params_base: usize,
    allocs: HashMap<ir::Value, usize>,
    current_offset: usize,
}

impl StackFrame {
    pub fn new() -> Self {
        Self {
            total: 0,
            params_base: 0,
            allocs: HashMap::new(),
            current_offset: 0,
        }
    }

    pub fn alloc_slot(&mut self, v: ir::Value) -> usize {
        // if let Some(&offset) = self.allocs.get(&v) {
        //     eprintln!("REUSE: {:?} -> offset {}", v, offset);
        //     return offset;
        // }

        let offset = self.params_base + self.current_offset;
        self.current_offset += 4;
        println!("current offset of sp: {}", self.current_offset);
        self.allocs.insert(v, offset);
        offset
    }

    pub fn alloc_array(&mut self, v: ir::Value, size: usize) -> usize {
        // if let Some(&offset) = self.allocs.get(&v) {
        //     return offset;
        // }
        
        let offset = self.params_base + self.current_offset;
        self.current_offset += size;
        println!("current offset of sp: {}", self.current_offset);
        self.allocs.insert(v, offset);
        offset
    }

    pub fn get_slot(&mut self, v: &ir::Value) -> usize {
        *self.allocs.get(&v).unwrap()
    }
}