use crate::code::Exp;
use Exp::*;

use std::str;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError {
    pub typ: PErrType,
    pub pos: usize
}
fn p_err<T>(typ: PErrType, pos: usize) -> Result<T, ParseError> {
    Err(ParseError { typ, pos })
}

#[derive(Debug, PartialEq, Eq)]
pub enum PErrType {
    EmptyCall,
    CloseEarly,
    NoClose,
    Incomplete,
    EmptyArgs,
    BadArgs,
    BadLet,
    Reserved
}
use PErrType::*;

#[derive(Debug, PartialEq, Eq)]
enum PCtx {
    Paren, Root, Fun, Let
}

const LAM_HI: u8 = b'\xce';
const LAM_LO: u8 = b'\xbb';
fn is_space(c: u8) -> bool {
    c == b' ' || c == b'\n' || c == b'\r' || c == b'\t'
}
fn skip_space(i: &mut usize, input: &[u8]) {
    while is_space(input[*i]) {
        *i += 1;
    }
}
fn is_reserved(i: &usize, input: &[u8]) -> bool {
    let c = input[*i];
    is_space(c) || c == b'(' || c == b')' || c == b'\\' || c == b'.' || c == b';' ||
        check_seq(i, input, &[LAM_HI, LAM_LO]) ||
        check_seq(i, input, b":=") ||
        (*i > 0 && is_space(input[*i-1]) && is_let(i, input))
}
fn is_let(i: &usize, input: &[u8]) -> bool {
    check_seq(i, input, b"let") && i + 3 < input.len() && is_space(input[*i+3])
}
fn check_seq(i: &usize, input: &[u8], seq: &[u8]) -> bool {
    i + seq.len() <= input.len() &&
        &input[*i..(i+seq.len())] == seq

}
fn push_call(ex: Option<Exp>, new: Exp) -> Option<Exp> {
    Some(match ex {
        Some(ex) => Call(Box::new(ex), Box::new(new)),
        None => new,
    })
}

pub fn parse(input: &str) -> Result<Exp, ParseError> {
    let mut i = 0;
    let inp = input.as_bytes();
    let ex = get_parse(&mut i, inp, PCtx::Root)?;
    if i == inp.len() {
        Ok(ex)
    } else {
        p_err(Incomplete, i)
    }
}

fn get_parse(i: &mut usize, input: &[u8], ctx: PCtx) -> Result<Exp, ParseError> {
    let mut ex: Option<Exp> = None;
    let mut closed = false;
    while *i < input.len() {
        match input[*i] {
            b'#' => {
                *i += 1;
                get_comment(i, input);
            }
            b'(' => {
                *i += 1;
                ex = push_call(ex, get_parse(i, input, PCtx::Paren)?);
            }
            b')' => {
                match ctx {
                    PCtx::Paren => {
                        closed = true;
                        *i += 1;
                        break;
                    }
                    PCtx::Root | PCtx::Let => {
                        return p_err(CloseEarly, *i);
                    }
                    PCtx::Fun => {
                        break;
                    }
                }
            }
            b';' => match ctx {
                PCtx::Let => {
                    *i += 1;
                    break;
                }
                PCtx::Root | PCtx::Paren => {
                    return p_err(Reserved, *i);
                }
                _ => {
                    break;
                }
            }
            b'\\' => {
                *i += 1;
                ex = push_call(ex, get_fun(i, input)?);
            }
            _ if check_seq(i, input, &[LAM_HI, LAM_LO]) => {
                *i += 2;
                ex = push_call(ex, get_fun(i, input)?);
            }
            _ if check_seq(i, input, b":=") => {
                return p_err(Reserved, *i)
            }
            _ if is_let(i, input) => {
                *i += 3;
                let (name, val) = get_let(i, input)?;
                let body = get_parse(i, input, PCtx::Fun)?;
                ex = push_call(ex, Call(
                    Box::new(Lamb(name, Box::new(body))),
                    Box::new(val)
                ));
            }
            ch if is_space(ch) => {
                *i += 1;
            }
            _ => {
                 ex = push_call(ex, Var(get_var(i, input)));
            }
        }
    }
    if ctx == PCtx::Paren && !closed {
        p_err(NoClose, *i)
    } else {
        match ex {
            Some(e) => Ok(e),
            None => p_err(EmptyCall, if ctx == PCtx::Paren { *i-1 } else { *i })
        }
    }
}

fn get_comment(i: &mut usize, input: &[u8]) {
    while *i < input.len() {
        if input[*i] == b'\n' {
            *i += 1;
            break;
        }
        *i += 1;
    }
}

fn get_var(i: &mut usize, input: &[u8]) -> String {
    let mut s: Vec<u8> = Vec::with_capacity(10);
    while *i < input.len() && !is_reserved(i, input) {
        s.push(input[*i]);
        *i += 1;
    }
    // s.shrink_to_fit();
    String::from_utf8(s).unwrap()
}

fn get_fun(i: &mut usize, input: &[u8]) -> Result<Exp, ParseError> {
    let mut args: Vec<String> = Vec::with_capacity(5);
    while *i < input.len() {
        match input[*i] {
            b'.' => {
                *i += 1;
                break;
            }
            ch if is_space(ch) => {
                *i += 1;
            }
            _ if is_reserved(i, input) => {
                return p_err(BadArgs, *i);
            }
            _ => {
                args.push(get_var(i, input));
            }
        }
    }
    
    if args.is_empty() {
        p_err(EmptyArgs, *i-1)
    } else if *i > input.len() {
        p_err(BadArgs, *i)
    } else {
        let bod = get_parse(i, input, PCtx::Fun)?;
        Ok(args.into_iter().rev().fold(bod, |r, a| Lamb(a, Box::new(r))))
    }
}

fn get_let(i: &mut usize, input: &[u8]) -> Result<(String, Exp), ParseError> {
    skip_space(i, input);
    let name = get_var(i, input);
    skip_space(i, input);
    if check_seq(i, input, b":=") {
        *i += 2;
    } else {
        return p_err(BadLet, *i)
    }
    skip_space(i, input);
    let val = get_parse(i, input, PCtx::Let)?;
    Ok((name, val))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_parse() {
        assert_eq!(parse("x"), Ok(Var("x".to_string())));
        assert_eq!(parse("a b"), Ok(
            Call(Box::new(Var("a".to_string())), Box::new(Var("b".to_string())))));
        assert_eq!(parse("\\z.z"), Ok(
            Lamb("z".to_string(), Box::new(Var("z".to_string())))));
    }
    #[test]
    fn parens() {
        assert_eq!(parse("x"), parse("(((x)))"));
        assert_eq!(parse("x y"), parse("(x y)"));
        assert_eq!(parse("(x) y"), parse("x (y)"));
        assert_eq!(parse("\\x.y x"), parse("(\\x. (y x))"));
    }
    #[test]
    fn calls() {
        assert_eq!(parse("x y z"),
            Ok(Call(
                Box::new(Call(
                    Box::new(Var("x".to_string())),
                    Box::new(Var("y".to_string()))
                )),
                Box::new(Var("z".to_string()))
            ))
        );
        assert_eq!(parse("x y z"), parse("(x y) z"));
        assert_eq!(parse("a b c d"), parse("(((a b) c) d)"));
        assert_ne!(parse("x y z"), parse("x (y z)"));
        assert_eq!(parse("x z (y z)"), parse("(x z) (y z)"));
    }
    #[test]
    fn lambdas() {
        assert_eq!(parse("\\x y. z"), parse("\\x. \\y. z"));
        assert_eq!(parse("a \\x. z"), parse("a (\\x. z)"));
        assert_eq!(parse("\\x y z. x z (y z)"), parse("\\x. \\y. \\z. (x z) (y z)"));
    }
    #[test]
    fn spaces() {
        assert_eq!(parse("  \\x .    x   "), parse("\\x.x"));
        assert_eq!(parse("   a        b    c  "), parse("a b c"));
    }
    #[test]
    fn unicode() {
        assert_eq!(parse("α"), Ok(Var("α".to_string())));
        assert_eq!(parse("λx.x"), parse("\\x.x"));
        assert_eq!(parse("yλx.x"), parse("y\\x.x"));
        assert_eq!(parse("ζλx.x"), parse("ζ \\x.x"));
    }
    #[test]
    fn comments() {
        assert_eq!(parse("\\x y. z # foo"), parse("\\x y. z"));
        assert_eq!(parse("\
(\\S K. S K K)       # substitute in combinators\n\
(\\x y z. x z (y z)) # S combinator: calls first argument on third, then\n\
                     # calls the result on the second applied to the third\n\
(\\x y. x)           # K combinator: returns first argument, ignores second\
        "), parse("(\\S K. S K K) (\\x y z. x z (y z)) (\\x y. x)"));
    }
    #[test]
    fn lets() {
        assert_eq!(parse("let x:=y; x x"), parse("(\\x. x x) y"));
        assert_eq!(parse("let x := y; let z := x w; x z"),
            parse("(\\x. (\\z. x z) (x w)) y"));
        assert_eq!(parse("\\y. let x := y y; x x"),
            parse("\\y. (\\x. x x) (y y)"));
        assert_eq!(parse("a let x := \\y. y; x x"),
            parse("a ((\\x. x x) \\y. y)"));
    }
    #[test]
    fn lets_not() {
        assert_eq!(parse("lettuce"), Ok(Var("lettuce".to_string())));
        assert_eq!(parse("islet"), Ok(Var("islet".to_string())));
        assert_eq!(parse("\\filets. filets"),
            Ok(Lamb("filets".to_string(), Box::new(Var("filets".to_string())))));
    }
    #[test]
    fn reserved() {
        assert_eq!(parse(":="), p_err(Reserved, 0));
        assert_eq!(parse("x := y"), p_err(Reserved, 2));
        assert_eq!(parse("f ;"), p_err(Reserved, 2));
        assert_eq!(parse("(a ; b)"), p_err(Reserved, 3));
        assert_eq!(parse(";"), p_err(Reserved, 0));
        assert_eq!(parse("a b; c"), p_err(Reserved, 3));
    }

    #[test]
    fn err_empty_calls() {
        assert_eq!(parse(""), p_err(EmptyCall, 0));
        assert_eq!(parse("()"), p_err(EmptyCall, 1));
        assert_eq!(parse("\\x."), p_err(EmptyCall, 3));
    }
    #[test]
    fn err_bad_args() {
        assert_eq!(parse("\\.x"), p_err(EmptyArgs, 1));
        assert_eq!(parse("\\"), p_err(EmptyArgs, 0));
        assert_eq!(parse("\\ "), p_err(EmptyArgs, 1));
        assert_eq!(parse("\\x\\"), p_err(BadArgs, 2));
        assert_eq!(parse("\\x)"), p_err(BadArgs, 2));
    }
    #[test]
    fn err_bad_args_unicode() {
        assert_eq!(parse("λ.α"), p_err(EmptyArgs, 2));
        assert_eq!(parse("λ"), p_err(EmptyArgs, 1));
        assert_eq!(parse("λ "), p_err(EmptyArgs, 2));
        assert_eq!(parse("λαλ"), p_err(BadArgs, 4));
    }
    #[test]
    fn err_no_close() {
        assert_eq!(parse("("), p_err(NoClose, 1));
        assert_eq!(parse("(x y z"), p_err(NoClose, 6));
        assert_eq!(parse("(x (y z)"), p_err(NoClose, 8));
    }
    #[test]
    fn err_close_early() {
        assert_eq!(parse(")"), p_err(CloseEarly, 0));
        assert_eq!(parse("x y) z"), p_err(CloseEarly, 3));
        assert_eq!(parse("(x y) z)"), p_err(CloseEarly, 7));
    }

}