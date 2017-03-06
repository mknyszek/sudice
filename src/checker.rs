use descriptor::{SudiceCode, SudiceExpression};

use std::ops::{Add, Sub, Mul, Div};
use std::vec::Vec;

enum CheckerValue {
    Scalar(i64),
    Vector(i64, i64),
}

impl CheckerValue {
    fn collapse(self) -> i64 {
        match self {
            CheckerValue::Scalar(s) => s,
            CheckerValue::Vector(l, s) => l * s,
        }
    }
}

pub fn semantic_check(d: &SudiceExpression) -> Result<(i64, i64), String> {
    let mut min_s: Vec<CheckerValue> = Vec::with_capacity(d.code.len());
    let mut min_tos = CheckerValue::Scalar(0);
    let mut max_s: Vec<CheckerValue> = Vec::with_capacity(d.code.len());
    let mut max_tos = CheckerValue::Scalar(0);
    macro_rules! nop {
        () => {{
            let _ = min_s.pop().unwrap();
            let _ = max_s.pop().unwrap();
        }}
    }

    macro_rules! arith_op {
        ($func:path) => {{
            let min_x = min_s.pop().unwrap().collapse();
            min_tos = CheckerValue::Scalar($func(min_tos.collapse(), min_x));
            let max_x = max_s.pop().unwrap().collapse();
            max_tos = CheckerValue::Scalar($func(max_tos.collapse(), max_x));
        }}
    }

    macro_rules! drop_op {
        () => {{
            let min_x = min_s.pop().unwrap().collapse();
            min_tos = match min_tos {
                CheckerValue::Scalar(_) => return Err("Attempted to drop scalar.".to_string()),
                CheckerValue::Vector(l, s) => {
                    if min_x >= l {
                        return Err("Attempted to drop too many values.".to_string());
                    }
                    CheckerValue::Vector(l - min_x, s)
                },
            };
            let max_x = max_s.pop().unwrap().collapse();
            max_tos = match max_tos {
                CheckerValue::Scalar(_) => panic!("Due to symmetry, this should not happen."),
                CheckerValue::Vector(l, s) => {
                    if max_x >= l {
                        panic!("Due to symmetry, this should not happen.");
                    }
                    CheckerValue::Vector(l - max_x, s)
                },
            };
        }}
    }
    for dc in d.code.iter() {
        match *dc {
            SudiceCode::Num(i) => {
                min_s.push(min_tos);
                min_tos = CheckerValue::Scalar(i);
                max_s.push(max_tos);
                max_tos = CheckerValue::Scalar(i);
            }, 
            SudiceCode::Add => arith_op!(i64::add),
            SudiceCode::Sub => arith_op!(i64::sub),
            SudiceCode::Mul => arith_op!(i64::mul),
            SudiceCode::Div => arith_op!(i64::div),
            SudiceCode::Roll => {
                let _ = min_s.pop().unwrap();
                min_tos = CheckerValue::Vector(min_tos.collapse(), 1);
                let max_x = max_s.pop().unwrap();
                max_tos = CheckerValue::Vector(max_tos.collapse(), max_x.collapse());
            },
            SudiceCode::Reroll => nop!(),
            SudiceCode::RerollLowest => nop!(),
            SudiceCode::RerollHighest => nop!(),
            SudiceCode::DropLowest => drop_op!(),
            SudiceCode::DropHighest => drop_op!(),
            SudiceCode::Ceil => {
                let min_x = min_s.pop().unwrap().collapse();
                min_tos = match min_tos {
                    CheckerValue::Scalar(s) => if s > min_x { CheckerValue::Scalar(min_x) } else { min_tos },
                    CheckerValue::Vector(l, s) => if s > min_x { CheckerValue::Vector(l, min_x) } else { min_tos },
                };
                let max_x = max_s.pop().unwrap().collapse();
                max_tos = match max_tos {
                    CheckerValue::Scalar(s) => if s > max_x { CheckerValue::Scalar(max_x) } else { max_tos },
                    CheckerValue::Vector(l, s) => if s > max_x { CheckerValue::Vector(l, max_x) } else { max_tos },
                };
            },
            SudiceCode::Floor => {
                let min_x = min_s.pop().unwrap().collapse();
                min_tos = match min_tos {
                    CheckerValue::Scalar(s) => if s < min_x { CheckerValue::Scalar(min_x) } else { min_tos },
                    CheckerValue::Vector(l, s) => if s < min_x { CheckerValue::Vector(l, min_x) } else { min_tos },
                };
                let max_x = max_s.pop().unwrap().collapse();
                max_tos = match max_tos {
                    CheckerValue::Scalar(s) => if s < max_x { CheckerValue::Scalar(max_x) } else { max_tos },
                    CheckerValue::Vector(l, s) => if s < max_x { CheckerValue::Vector(l, max_x) } else { max_tos },
                };
            },
            SudiceCode::BestOf(_) => nop!(),
            SudiceCode::WorstOf(_) => nop!(),
        }
    }
    Ok((min_tos.collapse(), max_tos.collapse()))
}
