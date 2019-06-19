use crate::code::Exp;
use Exp::*;

#[derive(Debug, PartialEq)]
enum Reduc {
    Left(Box<Reduc>),
    Right(Box<Reduc>),
    // Body(Box<Reduc>),
    // Alpha(String),
    Beta,
    // Eta,
    Irred
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{ parse, ParseError };

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
}