#[macro_use]
extern crate clap;
use clap::{ Arg, App };

use std::fs;
use std::io::{ stdin, stdout, Write };
use std::time::Instant;

mod code;
mod parser;
use parser::parse;
mod reduce;
use reduce::{ Strategy, reduce_iter, reduce_full, strat_norm, strat_byname };

fn main() {
    let matches = App::new("Lambda")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Evaluates lambda calculus terms")
        .arg(Arg::with_name("STRAT")
            .short("s")
            .long("strat")
            .takes_value(true)
            .possible_values(&["byname", "normal"])
            .default_value("normal")
            .help("Sets reduction order")
        )
        .arg(Arg::with_name("VERBOSE")
            .short("v")
            .long("verbose")
            .help("Lists individual reduction steps")
        )
        .arg(Arg::with_name("INPUT")
            .required(true)
            .help("Sets the source file to use, or if none given, launches a REPL")
        )
    .get_matches();
    let strat = match matches.value_of("STRAT") {
        Some("byname") => strat_byname,
        Some("normal") => strat_norm,
        _ => panic!("invalid strategy")
    };
    let verbose = matches.is_present("VERBOSE");
    if let Some(file) = matches.value_of("INPUT") {
        let inp = fs::read_to_string(file).expect("error loading file");
        run(&inp, strat, verbose);
    } else {
        loop {
            print!("> ");
            stdout().flush().expect("error flushing stdin");
            let mut inp = String::new();
            stdin().read_line(&mut inp).expect("error reading stdin");
            run(&inp, strat, verbose);
        }
    };
}

fn run(inp: &str, strat: Strategy, verbose: bool) {
    let now = Instant::now();
    let p = parse(&inp);
    println!("Parse time: {:.3}ms", now.elapsed().as_millis() as f64 * 1e-3);

    match p {
        Ok(ex) => {
            println!("{}", ex);
            let now = Instant::now();
            if verbose {
                for (red, ex) in reduce_iter(strat, ex) {
                    println!("=={}==>", red);
                    println!("{}", ex);
                }
            } else {
                println!("{}", reduce_full(strat, ex));
            }
            println!("Eval time: {:.6}s", now.elapsed().as_micros() as f64 * 1e-6);
        }
        Err(e) => {
            eprintln!("Parse error: {:?} at {:?}", e.typ, rowcol(e.pos, &inp));
        }
    }
}

fn rowcol(i: usize, s: &str) -> (usize, usize) {
    let mut row = 0;
    let mut col = 0;
    let mut j = 0;
    let b = s.as_bytes();
    while j <= i && j < b.len() {
        if b[j] == b'\n' {
            row += 1;
            col = 0;
        } else {
            col += 1;
        }
        j += 1;
    }
    (row, col)
}