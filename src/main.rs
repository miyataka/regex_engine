mod engine;
mod helper;

use helper::DynError;
use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
};

fn match_file(expr: &str, file: &str) -> Result<(), DynError> {
    let f = File::open(file)?;
    let reader = BufReader::new(f);

    engine::print(expr)?;
    println!();

    for line in reader.lines() {
        let line = line?;
        for (i, _) in line.char_indices() {
            if engine::do_matching(expr, &line[i..], true)? {
                println!("{line}");
                break;
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), DynError> {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 2 {
        eprintln!("usage: {} regex file", args[0]);
        return Err("invalid arguments".into());
    } else {
        match_file(&args[1], &args[2])?;
    }

    Ok(())
}

// unit test
#[cfg(test)]
mod tests {
    use crate::{
        engine::{do_matching, print},
        helper::{safe_add, SafeAdd},
    };

    #[test]
    fn test_safe_add() {
        let n: usize = 10;
        assert_eq!(Some(30), n.safe_add(&20));

        let n: usize = !0; // usize::MAX
        assert_eq!(None, n.safe_add(&1));

        let mut n: usize = 10;
        assert_eq!(safe_add(&mut n, &20, || ()).is_ok(), true);

        let mut n: usize = !0; // usize::MAX
        assert!(safe_add(&mut n, &1, || ()).is_err());
    }

    #[test]
    fn test_matching() {
        // parse error
        assert!(do_matching("+b", "bbb", true).is_err());
        assert!(do_matching("*b", "bbb", true).is_err());
        assert!(do_matching("|b", "bbb", true).is_err());
        assert!(do_matching("?b", "bbb", true).is_err());

        // parse success, match success
        assert!(do_matching("abc|def", "def", true).is_ok());
        assert!(do_matching("(abc)*", "abcabc", true).is_ok());
        assert!(do_matching("(ab|cd)+", "abcdcd", true).is_ok());
        assert!(do_matching("abc?", "ab", true).is_ok());

        // parse success, match fail
        assert!(!do_matching("abc|def", "efa", true).unwrap());
        assert!(!do_matching("(ab|cd)+", "", true).unwrap());
        assert!(do_matching("abc?", "acb", true).is_ok());
    }
}
