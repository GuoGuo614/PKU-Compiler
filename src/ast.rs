pub struct CompUnit {
  pub func_def: FuncDef,
}

pub enum Decl {
  Const(ConstDecl),
  Var(VarDecl)
}

pub struct ConstDecl {
  pub btype: BType,
  pub const_defs: Vec<ConstDef>,
}

pub struct BType {
  pub _type: String,
}

pub struct ConstDef {
  pub ident: String,
  pub const_val: ConstInitVal
}

pub struct ConstInitVal {
  pub const_exp: ConstExp,
}

pub struct VarDecl {
  pub b_type: BType,
  pub var_defs: Vec<VarDef>,
}

pub enum VarDef {
  Decl(String),
  Init(String, InitVal),
}

pub struct InitVal {
  pub exp: Exp,
}

pub struct ConstExp {
  pub exp: Exp,
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
  pub block_items: Vec<BlockItem>,
}

pub enum BlockItem {
  Decl(Decl),
  Stmt(Stmt),
}

pub enum Stmt {
  Assign(LVal, Exp),
  Return(Exp)
}

pub struct LVal {
  pub ident: String,
}

pub struct Exp {
  pub lor_exp: LOrExp,
}

pub enum UnaryExp {
  Primary(PrimaryExp),
  Positive(Box<UnaryExp>),
  Negative(Box<UnaryExp>),
  Not(Box<UnaryExp>),
}

pub enum PrimaryExp {
  Paren(Box<Exp>),
  LVal(LVal),
  Number(i32),
}

pub enum AddExp {
  Mul(MulExp),
  Add(Box<AddExp>, MulExp),
  Sub(Box<AddExp>, MulExp),
}

pub enum MulExp {
  Unary(UnaryExp),
  Mul(Box<MulExp>, UnaryExp),
  Div(Box<MulExp>, UnaryExp),
  Mod(Box<MulExp>, UnaryExp),
}

pub enum LOrExp {
  And(LAndExp),
  Or(Box<LOrExp>, LAndExp),
}

pub enum LAndExp {
  Eq(EqExp),
  And(Box<LAndExp>, EqExp),
}

pub enum EqExp {
  Rel(RelExp),
  Eq(Box<EqExp>, RelExp),
  Neq(Box<EqExp>, RelExp),
}

pub enum RelExp {
  Add(AddExp),
  Lt(Box<RelExp>, AddExp),
  Gt(Box<RelExp>, AddExp),
  Ge(Box<RelExp>, AddExp),
  Le(Box<RelExp>, AddExp),
}
