use crate::code::Exp;
use Exp::*;

use std::collections::HashSet;

#[derive(Debug, PartialEq)]
pub enum Reduc {
    Left(Box<Reduc>),
    Right(Box<Reduc>),
    // Body(Box<Reduc>),
    // Alpha(String),
    Beta,
    // Eta,
    Irred
}

// fn reduce_with(ex: Exp, red: Reduc) -> Exp {
//     match (ex, red) {
//         (Call(a, b), Reduc::Beta) => match *a {
//             Lamb(_, _) =>
//         }
//         (ex, Reduc::Irred) => ex,
//         (e, r) => panic!("bad reduction: {:?} on {}", r, e)
//     }
// }

pub fn red_basic(ex: Exp) -> Reduc {
    match ex {
        Call(a, b) => {
            match *a {
                Lamb(_, _) => Reduc::Beta,
                a => match red_basic(a) {
                    Reduc::Irred => match red_basic(*b) {
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

pub fn free_vars(ex: Exp) -> HashSet<String> {
    match ex {
        Var(n) => {
            let mut v = HashSet::new();
            v.insert(n);
            v
        }
        Call(a, b) => {
            let mut v = free_vars(*a);
            v.extend(free_vars(*b));
            v
        }
        Lamb(x, r) => {
            let mut v = free_vars(*r);
            v.remove(&x);
            v
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{ parse, ParseError };
    use std::collections::HashSet;
    use std::iter::FromIterator;

    #[test]
    fn irred_basic() -> Result<(), ParseError>{
        assert_eq!(red_basic(parse("x")?), Reduc::Irred);
        assert_eq!(red_basic(parse("a b")?), Reduc::Irred);
        assert_eq!(red_basic(parse("\\x.x")?), Reduc::Irred);
        assert_eq!(red_basic(parse("\\x. (\\y.y) z")?), Reduc::Irred);
        Ok(())
    }
    #[test]
    fn beta_basic() -> Result<(), ParseError> {
        assert_eq!(red_basic(parse("(\\x. x) y")?), Reduc::Beta);
        assert_eq!(red_basic(parse("(\\x. x) y z")?),
            Reduc::Left(Box::new(Reduc::Beta)));
        assert_eq!(red_basic(parse("z ((\\x. x) y)")?),
            Reduc::Right(Box::new(Reduc::Beta)));
        Ok(())
    }
    #[test]
    fn order_basic() -> Result<(), ParseError> {
        assert_eq!(red_basic(parse("(\\a. a) b ((\\x. x) y)")?),
            Reduc::Left(Box::new(Reduc::Beta)));
        assert_eq!(red_basic(parse("(\\x. (\\a. a) x) z w")?),
            Reduc::Left(Box::new(Reduc::Beta)));
        Ok(())
    }
    #[test]
    fn free() -> Result<(), ParseError> {
        assert_eq!(free_vars(parse("x")?),
            HashSet::from_iter(vec!["x".to_string()]));
        assert_eq!(free_vars(parse("x y z")?),
            HashSet::from_iter(vec!["x".to_string(), "y".to_string(), "z".to_string()]));
        assert_eq!(free_vars(parse("(\\x. x) y")?),
            HashSet::from_iter(vec!["y".to_string()]));
        assert_eq!(free_vars(parse("(\\x y. x)")?),
            HashSet::new());
        assert_eq!(free_vars(parse("(\\x y. x) y")?),
            HashSet::from_iter(vec!["y".to_string()]));
        Ok(())
    }
}