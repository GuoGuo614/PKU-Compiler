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
