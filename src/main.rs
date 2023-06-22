use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::helpers::DynError;

mod engine;
mod helpers;

fn main() -> Result<(), DynError> {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 2 {
        eprintln!("usage: {} regex file", args[0]); // 標準エラー出力の eprintln!
        return Err("Invalid arguments".into());
    } else {
        match_file(&args[1], &args[2])?;
    }
    Ok(())
}


fn match_file(expr: &str, file: &str)-> Result<(), DynError> {
    let f = File::open(file)?;
    let reader = BufReader::new(f);

    engine::print(expr)?;
    println!();

    for line in reader.lines() {
        let line = line?;
        for (i, _) in line.char_indices() {
            if engine::do_matching(expr, &line[i..], true)? {
                println!("hit!!!: {line}");
                break;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::mem::take;

    #[test]
    fn take_sample() {
        let mut n = Some(10);
        let v = take(&mut n);
        println!("n = {:?}, v = {:?}", n, v);
    }

    #[test]
    fn unicode() {
        let s = "aはAの小文字".as_bytes();
        for i in s {
            print!("{:x}", i);
        }
        println!();
    }
}