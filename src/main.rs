use std::fs;
use std::io::{ stdin, stdout, Read, Write };
use std::env;
use std::time::Instant;

mod code;
mod parser;
use parser::parse;
mod reduce;
use reduce::{ reduce_iter, strat_byname };

fn main() {
    let args: Vec<String> = env::args().collect();
    let inp = if args.len() >= 2 {
        fs::read_to_string(&args[1]).expect("error loading file")
    } else {
        print!("> ");
        stdout().flush().expect("what the fuck");
        let mut buf = String::new();
        stdin().read_to_string(&mut buf).expect("error reading stdin");
        buf
    };

    let now = Instant::now();
    let p = parse(&inp);
    println!("Parse time: {:.3}ms", now.elapsed().as_micros() as f64 * 1e-3);

    match p {
        Ok(ex) => {
            println!("{}", ex);
            let now = Instant::now();
            for (red, ex) in reduce_iter(strat_byname, ex) {
                println!("=={}==>", red);
                println!("{}", ex);
            }
            println!("Eval time: {:.3}ms", now.elapsed().as_micros() as f64 * 1e-3);
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