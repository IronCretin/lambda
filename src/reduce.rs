use crate::code::Exp;
use Exp::*;

use std::fmt;
use std::iter::Iterator;
// use std::mem::swap;

#[derive(Debug, PartialEq)]
pub enum Reduc {
    Left(Box<Reduc>),
    Right(Box<Reduc>),
    // Body(Box<Reduc>),
    Beta,
    // Eta,
    Irred
}
impl fmt::Display for Reduc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Reduc::Left(r) => write!(f, "({} _)", r),
            Reduc::Right(r) => write!(f, "(_ {})", r),
            // Reduc::Body(r) => write!(f, "(\\_. {})", r),
            Reduc::Beta => write!(f, "β"),
            // Reduc::Eta => write!(f, "η"),
            Reduc::Irred => write!(f, "-"),
        }
    }
}

fn reduce_with(ex: Exp, red: &Reduc) -> Exp {
    match (ex, red) {
        (Call(a, b), Reduc::Beta) => match *a {
            Lamb(x, r) => sub(*r, &x, *b),
            a => panic!("bad beta reduction: lhs {}", a)
        }
        (Call(a, b), red) => match red {
            Reduc::Left(r) => Call(Box::new(reduce_with(*a, r)), b),
            Reduc::Right(r) => Call(a, Box::new(reduce_with(*b, r))),
            Reduc::Irred => Call(a, b),
            red => panic!("bad reduction: {:?} on {}", red, Call(a, b))
        }
        (ex, Reduc::Irred) => ex,
        (ex, red) => panic!("bad reduction: {:?} on {}", red, ex)
    }
}

pub fn reduce_step(strat: fn(&Exp) -> Reduc, ex: Exp) -> (Reduc, Exp) {
    let red = strat(&ex);
    let ex = reduce_with(ex, &red);
    (red, ex)
}

pub fn reduce_full(strat: fn(&Exp) -> Reduc, ex: Exp) -> Exp {
    let mut red: Reduc;
    let mut ex = ex;
    loop {
        let t = reduce_step(strat, ex);
        red = t.0;
        ex = t.1;
        if red == Reduc::Irred {
            return ex;
        }
    }
}

pub struct ReducIter {
    strat: fn(&Exp) -> Reduc,
    ex: Exp
}
impl Iterator for ReducIter {
    type Item = (Reduc, Exp);
    fn next(&mut self) -> Option<(Reduc, Exp)> {
        let red = (self.strat)(&self.ex);
        self.ex = reduce_with(self.ex.clone(), &red);
        match red {
            Reduc::Irred => None,
            red => Some((red, (&self.ex).clone()))
        }
    }
}

pub fn reduce_iter(strat: fn(&Exp) -> Reduc, ex: Exp) -> ReducIter {
    ReducIter { strat, ex }
}

pub fn strat_byname(ex: &Exp) -> Reduc {
    match ex {
        Call(a, b) => {
            match **a {
                Lamb(_, _) => Reduc::Beta,
                _ => match strat_byname(a) {
                    Reduc::Irred => match strat_byname(b) {
                        Reduc::Irred => Reduc::Irred,
                        r => Reduc::Right(Box::new(r))
                    }
                    r => Reduc::Left(Box::new(r))
                }
            }
        }
        _ => Reduc::Irred
    }
}

pub fn free_in(var: &str, ex: &Exp) -> bool {
    match ex {
        Var(n) => {
            var == n
        }
        Call(a, b) => {
            free_in(var, a) || free_in(var, b)
        }
        Lamb(x, r) => {
            var != x && free_in(var, r)
        }
    }
}

pub fn sub(ex: Exp, name: &str, new: Exp) -> Exp {
    match ex {
        Var(n) => if name == n {
            new
        } else {
            Var(n)
        }
        Call(a, b) => Call(Box::new(sub(*a, name, new.clone())), Box::new(sub(*b, name, new))),
        Lamb(x, r) => if name == x {
            Lamb(x, r)
        } else if free_in(&x, &new) {
            let mut x_new = x.clone();
            x_new.push('\'');
            sub(alpha(Lamb(x, r), x_new), name, new)
        } else {
            Lamb(x, Box::new(sub(*r, name, new)))
        }
    }
}

fn alpha(ex: Exp, new: String) -> Exp {
    if let Lamb(x, r) = ex {
        Lamb(new.clone(), Box::new(sub(*r, &x, Var(new))))
    }
    else {
        panic!("{} is not a lambda expression", ex)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{ parse, ParseError };

    #[test]
    fn reductions() -> Result<(), ParseError> {
        assert_eq!(reduce_with(parse("x")?, &Reduc::Irred), parse("x")?);
        assert_eq!(reduce_with(parse("(\\x. y x) z")?, &Reduc::Irred), parse("(\\x. y x) z")?);
        assert_eq!(reduce_with(parse("(\\x. y x) z")?, &Reduc::Beta), parse("y z")?);
        assert_eq!(reduce_with(parse("((\\x z. y x z) z)")?, &Reduc::Beta), parse("\\z'. y z z'")?);
        assert_eq!(reduce_with(parse("(\\a. a) b ((\\x. x) y)")?, &Reduc::Left(Box::new(Reduc::Beta))),
            parse("b ((\\x. x) y)")?);
        assert_eq!(reduce_with(parse("(\\a. a) b ((\\x. x) y)")?, &Reduc::Right(Box::new(Reduc::Beta))),
            parse("(\\a. a) b y")?);
        Ok(())
    }
    #[test]
    fn step_byname() -> Result<(), ParseError> {
        assert_eq!(reduce_step(strat_byname, parse("x")?), (Reduc::Irred, parse("x")?));
        assert_eq!(reduce_step(strat_byname, parse("(\\a. a) b ((\\x. x) y)")?),
            (Reduc::Left(Box::new(Reduc::Beta)), parse("b ((\\x. x) y)")?));
        Ok(())
    }
    #[test]
    fn skk_step_byname() -> Result<(), ParseError> {
        assert_eq!(reduce_step(strat_byname, parse("(\\S K. S K K) (\\x y z. x z (y z)) (\\x y. x)")?),
            (Reduc::Left(Box::new(Reduc::Beta)), parse("(\\K. (\\x y z. x z (y z)) K K) (\\x y. x)")?));

        assert_eq!(reduce_step(strat_byname, parse("(\\K. (\\x y z. x z (y z)) K K) (\\x y. x)")?),
            (Reduc::Beta, parse("(\\x y z. x z (y z)) (\\x y. x) (\\x y. x)")?));

        assert_eq!(reduce_step(strat_byname, parse("(\\x y z. x z (y z)) (\\x y. x) (\\x y. x)")?),
            (Reduc::Left(Box::new(Reduc::Beta)), parse("(\\y z. (\\x y. x) z (y z)) (\\x y. x)")?));

        assert_eq!(reduce_step(strat_byname, parse("(\\y z. (\\x y. x) z (y z)) (\\x y. x)")?),
            (Reduc::Beta, parse("\\z. (\\x y. x) z ((\\x y. x) z)")?));

        assert_eq!(reduce_step(strat_byname, parse("\\z. (\\x y. x) z ((\\x y. x) z)")?),
            (Reduc::Irred, parse("\\z. (\\x y. x) z ((\\x y. x) z)")?));
        Ok(())
    }
    #[test]
    fn skk_iter_byname() -> Result<(), ParseError> {
        let steps: Vec<(Reduc, Exp)> = reduce_iter(strat_byname, parse("(\\S K. S K K) (\\x y z. x z (y z)) (\\x y. x)")?).into_iter().collect();
        assert_eq!(steps,
            vec![
                (Reduc::Left(Box::new(Reduc::Beta)), parse("(\\K. (\\x y z. x z (y z)) K K) (\\x y. x)")?),
                (Reduc::Beta, parse("(\\x y z. x z (y z)) (\\x y. x) (\\x y. x)")?),
                (Reduc::Left(Box::new(Reduc::Beta)), parse("(\\y z. (\\x y. x) z (y z)) (\\x y. x)")?),
                (Reduc::Beta, parse("\\z. (\\x y. x) z ((\\x y. x) z)")?)
            ]);
        Ok(())
    }
    #[test]
    fn skk_full_byname() -> Result<(), ParseError> {
        assert_eq!(reduce_full(strat_byname, parse("(\\S K. S K K) (\\x y z. x z (y z)) (\\x y. x)")?),
            parse("\\z. (\\x y. x) z ((\\x y. x) z)")?);
        assert_eq!(reduce_full(strat_byname, parse("(\\S K. S K K) (\\x y z. x z (y z)) (\\x y. x) a")?),
            parse("a")?);
        Ok(())
    }
    #[test]
    fn skk_steps_byname() -> Result<(), ParseError> {
        assert_eq!(reduce_full(strat_byname, parse("(\\S K. S K K) (\\x y z. x z (y z)) (\\x y. x)")?),
            parse("\\z. (\\x y. x) z ((\\x y. x) z)")?);
        assert_eq!(reduce_full(strat_byname, parse("(\\S K. S K K) (\\x y z. x z (y z)) (\\x y. x) a")?),
            parse("a")?);
        Ok(())
    }
    #[test]
    fn irstrat_byname() -> Result<(), ParseError> {
        assert_eq!(strat_byname(&parse("x")?), Reduc::Irred);
        assert_eq!(strat_byname(&parse("a b")?), Reduc::Irred);
        assert_eq!(strat_byname(&parse("\\x.x")?), Reduc::Irred);
        assert_eq!(strat_byname(&parse("\\x. (\\y.y) z")?), Reduc::Irred);
        Ok(())
    }
    #[test]
    fn beta_byname() -> Result<(), ParseError> {
        assert_eq!(strat_byname(&parse("(\\x. x) y")?), Reduc::Beta);
        assert_eq!(strat_byname(&parse("(\\x. x) y z")?),
            Reduc::Left(Box::new(Reduc::Beta)));
        assert_eq!(strat_byname(&parse("z ((\\x. x) y)")?),
            Reduc::Right(Box::new(Reduc::Beta)));
        Ok(())
    }
    #[test]
    fn order_byname() -> Result<(), ParseError> {
        assert_eq!(strat_byname(&parse("(\\a. a) b ((\\x. x) y)")?),
            Reduc::Left(Box::new(Reduc::Beta)));
        assert_eq!(strat_byname(&parse("(\\x. (\\a. a) x) z w")?),
            Reduc::Left(Box::new(Reduc::Beta)));
        Ok(())
    }
    #[test]
    fn free() -> Result<(), ParseError> {
        assert!(free_in("x", &parse("x")?));
        assert!(free_in("y", &parse("x y z")?));
        assert!(free_in("y", &parse("(\\x. x) y")?));
        assert!(!free_in("x", &parse("(\\x. x) y")?));
        assert!(!free_in("y", &parse("(\\x y. x)")?));
        assert!(free_in("y", &parse("(\\x y. x) y")?));
        Ok(())
    }
    #[test]
    fn substitution() -> Result<(), ParseError> {
        assert_eq!(sub(parse("x")?, "x", parse("y")?), parse("y")?);
        assert_eq!(sub(parse("x y")?, "x", parse("z")?), parse("z y")?);
        assert_eq!(sub(parse("\\x. x z")?, "z", parse("w")?), parse("\\x. x w")?);
        assert_eq!(sub(parse("\\x. x")?, "x", parse("z")?), parse("\\x. x")?);
        assert_eq!(sub(parse("\\x. x z")?, "z", parse("x")?), parse("\\x'. x' x")?);
        Ok(())
    }
}