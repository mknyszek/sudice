use descriptor::{SudiceCode, SudiceExpression};

use std::cmp::PartialOrd;
use std::i64;
use std::ops::{Add, Sub, Mul, Div};
use std::vec::Vec;

#[derive(Clone, Copy)]
enum CheckerValue {
    Scalar(i64),
    Vector(i64, i64),
}

impl CheckerValue {
    fn collapse(&self) -> i64 {
        match *self {
            CheckerValue::Scalar(s) => s,
            CheckerValue::Vector(l, s) => l * s,
        }
    }
}

struct CheckerState {
    pub min_s: Vec<CheckerValue>,
    pub min_tos: CheckerValue,
    pub max_s: Vec<CheckerValue>,
    pub max_tos: CheckerValue,
}

impl CheckerState {
    fn new(capacity: usize) -> CheckerState {
        CheckerState {
            min_s: Vec::with_capacity(capacity),
            min_tos: CheckerValue::Scalar(0),
            max_s: Vec::with_capacity(capacity),
            max_tos: CheckerValue::Scalar(0)
        }
    }

    fn push(&mut self, min: CheckerValue, max: CheckerValue) {
        self.min_s.push(self.min_tos);
        self.min_tos = min;
        self.max_s.push(self.max_tos);
        self.max_tos = max;
    }

    fn pop(&mut self) {
        self.min_tos = self.min_s.pop().unwrap();
        self.max_tos = self.max_s.pop().unwrap();
    }

    fn nop(&mut self) {
        let _ = self.min_s.pop().unwrap();
        let _ = self.max_s.pop().unwrap();
    }
}

pub fn semantic_check(d: &SudiceExpression) -> Result<(i64, i64), String> {
    let mut state = CheckerState::new(d.code.len());
    semantic_check_with(d, 0, false, &mut state)?;
    Ok((state.min_tos.collapse(), state.max_tos.collapse()))
}

fn semantic_check_with(d: &SudiceExpression, start: usize, until_jump: bool, state: &mut CheckerState) -> Result<(), String> {
    macro_rules! arith_op {
        ($func:path) => {{
            let min_x = state.min_s.pop().unwrap().collapse();
            let max_x = state.max_s.pop().unwrap().collapse();
            state.min_tos = CheckerValue::Scalar($func(state.min_tos.collapse(), min_x));
            state.max_tos = CheckerValue::Scalar($func(state.max_tos.collapse(), max_x));
        }}
    }

    macro_rules! arith_op_inv {
        ($func:path) => {{
            let min_x = state.min_s.pop().unwrap().collapse();
            let max_x = state.max_s.pop().unwrap().collapse();
            state.min_tos = CheckerValue::Scalar($func(state.min_tos.collapse(), max_x));
            state.max_tos = CheckerValue::Scalar($func(state.max_tos.collapse(), min_x));
        }}
    }
    macro_rules! drop_op {
        () => {{
            let min_x = state.min_s.pop().unwrap().collapse();
            state.min_tos = match state.min_tos {
                CheckerValue::Scalar(_) => return Err("Attempted to drop scalar.".to_string()),
                CheckerValue::Vector(l, s) => {
                    if min_x >= l {
                        return Err("Attempted to drop too many values.".to_string());
                    }
                    CheckerValue::Vector(l - min_x, s)
                },
            };
            let max_x = state.max_s.pop().unwrap().collapse();
            state.max_tos = match state.max_tos {
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
    macro_rules! cap_op {
        ($func:path) => {{
            let min_x = state.min_s.pop().unwrap().collapse();
            state.min_tos = match state.min_tos {
                CheckerValue::Scalar(s) => if $func(&s, &min_x) { CheckerValue::Scalar(min_x) } else { state.min_tos },
                CheckerValue::Vector(l, s) => if $func(&s, &min_x) { CheckerValue::Vector(l, min_x) } else { state.min_tos },
            };
            let max_x = state.max_s.pop().unwrap().collapse();
            state.max_tos = match state.max_tos {
                CheckerValue::Scalar(s) => if $func(&s, &max_x) { CheckerValue::Scalar(max_x) } else { state.max_tos },
                CheckerValue::Vector(l, s) => if $func(&s, &max_x) { CheckerValue::Vector(l, max_x) } else { state.max_tos },
            };
        }}
    }
    let mut dcp = start;
    while dcp < d.code.len() {
        match d.code[dcp] {
            SudiceCode::Num(i) => state.push(CheckerValue::Scalar(i), CheckerValue::Scalar(i)),
            SudiceCode::Add => arith_op!(i64::add),
            SudiceCode::Sub => arith_op_inv!(i64::sub),
            SudiceCode::Mul => arith_op!(i64::mul),
            SudiceCode::Div => arith_op_inv!(i64::div),
            SudiceCode::Roll => {
                let _ = state.min_s.pop().unwrap();
                state.min_tos = CheckerValue::Vector(state.min_tos.collapse(), 1);
                let max_x = state.max_s.pop().unwrap();
                state.max_tos = CheckerValue::Vector(state.max_tos.collapse(), max_x.collapse());
            },
            SudiceCode::Reroll => state.nop(),
            SudiceCode::RerollLowest => state.nop(),
            SudiceCode::RerollHighest => state.nop(),
            SudiceCode::DropLowest => drop_op!(),
            SudiceCode::DropHighest => drop_op!(),
            SudiceCode::Ceil => cap_op!(i64::gt),
            SudiceCode::Floor => cap_op!(i64::lt),
            SudiceCode::BestOf(_) => state.nop(),
            SudiceCode::WorstOf(_) => state.nop(),
            SudiceCode::Select(ref offsets) => {
                let len = offsets.len();
                let first_min = state.min_tos.collapse();
                let first_max = state.max_tos.collapse();
                let (mut min, mut max) = if first_min < 1 || first_max > (len+1) as i64 {
                    (first_min, first_max)
                } else {
                    (i64::MAX, i64::MIN)
                };
                macro_rules! recursive_check {
                    ($e:expr) => {{
                        semantic_check_with(d, dcp + $e + 1, true, state)?;
                        let new_min = state.min_tos.collapse();
                        let new_max = state.max_tos.collapse();
                        if new_min < min { min = new_min; }
                        if new_max > max { max = new_max; }
                        state.pop();
                    }}
                }
                recursive_check!(0);
                for i in 0..len-1 {
                    recursive_check!(offsets[i]);
                }
                state.push(CheckerValue::Scalar(min), CheckerValue::Scalar(max));
                dcp += offsets[len-1];
            },
            SudiceCode::Jump(_) => if until_jump {
                return Ok(());
            } else {
                panic!("Error: Illegal bytecode sequence: Should not reach jump!");
            },
        }
        dcp += 1;
    }
    Ok(())
}
