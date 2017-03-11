#![recursion_limit = "200"]
#[macro_use]
extern crate pest;
extern crate rand;

mod descriptor;
mod parser;
mod checker;
mod interpreter;
mod estimator;

use parser::Rdp;

use pest::prelude::*;

#[cfg(test)]
mod test {
    use parser::Rdp;
    use checker;
    use estimator;
    use pest::prelude::StringInput;

    // Test helper function.
    //
    // Do not use any expr with an EV/SD of 0 for testing since results
    // are based on percent error off actual EV/SD.
    fn check_expr(expr: &'static str, ev: f64, sd: f64, range: i64) {
        let mut parser = Rdp::new(StringInput::new(expr));
        assert!(parser.expr());
        let code = parser.compile();
        match checker::semantic_check(&code) {
            Ok((min, max)) => {
                let results = estimator::estimate(&code, min, max);
                assert!(results.max - results.min + 1 == range);
                if ev < 0.0 {
                    assert!(results.ev >= ev * 1.02 && results.ev <= ev * 0.98);
                } else {
                    assert!(results.ev <= ev * 1.02 && results.ev >= ev * 0.98);
                }
                assert!(results.sd <= sd * 1.02 && results.sd >= sd * 0.98);
            },
            Err(s) => panic!("Semantic check failed: {}", s),
        }
    }

    #[test]
    fn simple_rolls() {
        check_expr("1d6", 3.5, 1.708, 6);
        check_expr("3d6", 10.5, 2.958, 16);
    }

    #[test]
    fn arithmetic() {
        check_expr("3 + 7", 10.0, 0.0, 1);
        check_expr("3 - 7", -4.0, 0.0, 1);
        check_expr("3 * 7", 21.0, 0.0, 1);
        check_expr("21 / 7", 3.0, 0.0, 1);
    }

    #[test]
    fn rolls_with_sum() {
        check_expr("2d8 - 3", 6.0, 3.240, 15);
        check_expr("2+4d4", 12.0, 2.236, 13);
        check_expr("1d20-1d20+20", 20.0, 8.15, 39);
    }

    #[test]
    fn rolls_with_mult() {
        check_expr("10*1d2+2d6-11", 11.0, 5.55, 21);
        check_expr("1d6*1d6", 12.25, 8.94, 36);
        check_expr("1d6/1d6", 1.14, 1.46, 7);
    }

    #[test]
    fn rolls_with_drop() {
        check_expr("3d6\\h1", 5.54, 2.215, 11);
        check_expr("4d6\\l1", 12.24, 2.847, 16);
    }

    #[test]
    fn rolls_with_iteration() {
        check_expr("1d20b2", 13.82, 4.71, 20);
        check_expr("1d20w2", 7.17, 4.71, 20);
    }
}

fn main() {
    let mut parser = Rdp::new(StringInput::new("[1d2 ? ((1d10+10)b2) (1d10w2)]"));
    assert!(parser.expr());
    let code = parser.compile();
    match checker::semantic_check(&code) {
        Ok((min, max)) => println!("{}", estimator::estimate(&code, min, max)),
        Err(s) => println!("Error: {}", s),
    }
}
