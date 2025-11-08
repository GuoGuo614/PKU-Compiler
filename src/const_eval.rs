use crate::ast::*;
use crate::symbol::SymbolTable;

pub trait EvalConst {
    fn eval(&self, sym: &SymbolTable) -> i32;
}

impl EvalConst for Exp {
    fn eval(&self, sym: &SymbolTable) -> i32 { self.lor_exp.eval(sym) }
}
impl EvalConst for LOrExp {
    fn eval(&self, sym: &SymbolTable) -> i32 {
        match self {
            LOrExp::And(a) => a.eval(sym),
            LOrExp::Or(l, r) => ((l.eval(sym) != 0) || (r.eval(sym) != 0)) as i32,
        }
    }
}
impl EvalConst for LAndExp {
    fn eval(&self, sym: &SymbolTable) -> i32 {
        match self {
            LAndExp::Eq(e) => e.eval(sym),
            LAndExp::And(l, r) => ((l.eval(sym) != 0) && (r.eval(sym) != 0)) as i32,
        }
    }
}
impl EvalConst for EqExp {
    fn eval(&self, sym: &SymbolTable) -> i32 {
        match self {
            EqExp::Rel(r) => r.eval(sym),
            EqExp::Eq(l, r) => (l.eval(sym) == r.eval(sym)) as i32,
            EqExp::Neq(l, r) => (l.eval(sym) != r.eval(sym)) as i32,
        }
    }
}
impl EvalConst for RelExp {
    fn eval(&self, sym: &SymbolTable) -> i32 {
        match self {
            RelExp::Add(a) => a.eval(sym),
            RelExp::Lt(l, r) => (l.eval(sym) <  r.eval(sym)) as i32,
            RelExp::Le(l, r) => (l.eval(sym) <= r.eval(sym)) as i32,
            RelExp::Gt(l, r) => (l.eval(sym) >  r.eval(sym)) as i32,
            RelExp::Ge(l, r) => (l.eval(sym) >= r.eval(sym)) as i32,
        }
    }
}
impl EvalConst for AddExp {
    fn eval(&self, sym: &SymbolTable) -> i32 {
        match self {
            AddExp::Mul(m) => m.eval(sym),
            AddExp::Add(l, r) => l.eval(sym) + r.eval(sym),
            AddExp::Sub(l, r) => l.eval(sym) - r.eval(sym),
        }
    }
}
impl EvalConst for MulExp {
    fn eval(&self, sym: &SymbolTable) -> i32 {
        match self {
            MulExp::Unary(u) => u.eval(sym),
            MulExp::Mul(l, r) => l.eval(sym) * r.eval(sym),
            MulExp::Div(l, r) => l.eval(sym) / r.eval(sym),
            MulExp::Mod(l, r) => l.eval(sym) % r.eval(sym),
        }
    }
}
impl EvalConst for UnaryExp {
    fn eval(&self, sym: &SymbolTable) -> i32 {
        match self {
            UnaryExp::Primary(p) => p.eval(sym),
            UnaryExp::Positive(u) =>  u.eval(sym),
            UnaryExp::Negative(u) => -u.eval(sym),
            UnaryExp::Not(u) => (u.eval(sym) == 0) as i32,
        }
    }
}
impl EvalConst for PrimaryExp {
    fn eval(&self, sym: &SymbolTable) -> i32 {
        match self {
            PrimaryExp::Number(n) => *n,
            PrimaryExp::Paren(e) => e.eval(sym),
            PrimaryExp::LVal(lv) => sym.get_const(&lv.ident).copied().expect("const not found"),
        }
    }
}