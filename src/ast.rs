pub struct CompUnit {
  pub func_def: FuncDef,
}

pub struct FuncDef {
  pub func_type: FuncType,
  pub ident: String,
  pub block: Block,
}

pub struct FuncType {
  pub _type: String,
}

pub struct Block {
  pub stmt: Stmt,
}

pub enum Stmt {
  Return(Exp)
}

pub struct Exp {
  pub unary_exp: UnaryExp,
}

pub enum UnaryExp {
  Primary(PrimaryExp),
  Positive(Box<UnaryExp>),
  Negative(Box<UnaryExp>),
  Not(Box<UnaryExp>),
}

pub enum PrimaryExp {
  Paren(Box<Exp>),
  Number(i32),
}

// pub enum UnaryOp {
//   Plus,
//   Minus,
//   Not,
// }
