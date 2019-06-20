use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Exp {
    Var(String),
    Call(Box<Exp>, Box<Exp>),
    Lamb(String, Box<Exp>),
}
impl fmt::Display for Exp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Exp::Var(n) =>
                if f.alternate() {
                    write!(f, ". {}", n)
                } else {
                    write!(f, "{}", n)
                },
            Exp::Call(a, b) => 
                if f.alternate() {
                    write!(f, ". {:+} {:-}", a, b)
                } else if f.sign_minus() {
                    write!(f, "({:+} {:-})", a, b)
                } else {
                    write!(f, "{:+} {:-}", a, b)
                },
            Exp::Lamb(v, r) => 
                if f.alternate() {
                    write!(f, " {}{:#}", v, r)
                } else if f.sign_plus() || f.sign_minus() {
                    write!(f, "(λ{}{:#})", v, r)
                } else {
                    write!(f, "λ{}{:#}", v, r)
                }
        }
    }
}