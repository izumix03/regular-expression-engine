use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::helpers::DynError;

mod engine;
mod helpers;

// cargo run "abc*" regex.tex
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
    use crate::engine::do_matching;
    use crate::helpers::{safe_add, SafeAdd};

    #[test]
    fn test_safe_add() {
        let u = 10;
        assert_eq!(Some(30), u.safe_add(&20));

        let u = !0; // 2^64 -1 (64bit cpuの場合) 18446744073709551615
        assert_eq!(None, u.safe_add(&1));

        let u = 18446744073709551614; // ギリギリ足せる
        assert_eq!(Some(18446744073709551615), u.safe_add(&1));

        // エラーのクロージャ渡してもOK
        let mut n = 10;
        assert!(safe_add(&mut n, &20, || ()).is_ok());

        // エラーの確認
        let mut n = !0;
        assert!(safe_add(&mut n, &1, || ()).is_err());
    }

    #[test]
    fn test_matching() {
        // パースエラー
        assert!(do_matching("+b", "bbb", true).is_err());
        assert!(do_matching("*b", "bbb", true).is_err());
        assert!(do_matching("|b", "bbb", true).is_err());
        assert!(do_matching("?b", "bbb", true).is_err());

        // マッチ成功
        assert!(do_matching("abc|def", "def", true).unwrap());
        assert!(do_matching("(abc)*", "abcabc", true).unwrap());
        assert!(do_matching("(ab|cd)+", "abcdcd", true).unwrap());
        assert!(do_matching("abc?", "ab", true).unwrap());

        // マッチしない
        assert!(!do_matching("abc|def", "efa", true).unwrap());
        assert!(!do_matching("(ab|cd)+", "", true).unwrap());
        assert!(!do_matching("abc?", "acb", true).unwrap());
    }
}