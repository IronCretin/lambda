# Lambda

A simple lambda calculus interpreter.

## Features

Parsing and reduction of lambda calculus expressions, including unicode support and syntax sugar:

```plain
(\S K. S K K) (\x y z. x z (y z)) (\x y. x)
    ==
((λS. λK. (S K) K) (λx λy. λz. (x z) (y z))) (λx. λy. x)
```

Can evaluate from files, or run in a REPL.

Option to list reduction steps:

```plain
(\S K. S K K) (\x y z. x z (y z)) (\x y. x)
==(β _)==>
(\K. (\x y z. x z (y z)) K K) (\x y. x)
==β==>
(\x y z. x z (y z)) (\x y. x) (\x y. x)
==(β _)==>
(\y z. (\x y. x) z (y z)) (\x y. x)
==β==>
\z. (\x y. x) z ((\x y. x) z)
==(\. (β _))==>
\z. (\y. z) ((\x y. x) z)
==(\. β)==>
\z. z
```

## Installation

Either download a binary from the [releases](https://github.com/IronCretin/lambda/releases) page, or build manually (requires [rust](https://www.rust-lang.org/tools/install)):

```bash
$ git clone https://github.com/IronCretin/lambda.git
$ cd lambda
$ cargo build
$ cargo run -- -v
Lambda v0.2.0-alpha
λ>
```
>
> NOTE: `cargo run` requires flags to be passed behind `--` in order to pass them to the executable.

### Dependencies

- [Clap](https://crates.io/crates/clap) - command line argument parser.

## Usage

```plain
USAGE:
    lambda [FLAGS] [OPTIONS] [INPUT]

FLAGS:
    -l, --list       Lists individual reduction steps
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -s, --strat <STRAT>    Sets reduction order [default: normal]  [possible values: byname, normal]

ARGS:
    <INPUT>    Sets the source file to use, or if none given, launches a REPL
```

## Syntax

Lambda expressions consist of variables, applications (calls), and lambda abstractions (calls)

```bnf
<expression> ::= <name>
               | <expression> <expression>
               | ("λ" | "\") <name> "." <expression>
               | let <name> ":=" <expression>; <expression>
<name> ::= <one or more of any characters except one of λ \ . # ; := ( )>
```

Applications associate to the left, and the body of a lambda abstraction or `let` statement extends as far to the right as possible, with explicit grouping indicated by parentheses.

```plain
a b c == ((a b) c) != (a (b c))
f \x. x y == f (\x. x y) != (f (\x. x)) y
```

`let` statements are sugar for lambda applications:
```plain
let x := y; x x == (\x. x x) y
```

Whitespace is disregarded, and comments are started by `#` and continue until the end of the line.

## License

Lambda is distributed under the terms of the GNU GPL v3
