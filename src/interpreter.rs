use descriptor::*;

use rand::distributions::{IndependentSample, Range};
use rand::ThreadRng;

use std::ops::{Add, Sub, Mul, Div};

use std::io;
use std::io::Write;
use std::vec::Vec;

pub fn compute_range(d: &Descriptor) -> (i64, i64) {
    let mut min_s = Vec::with_capacity(d.code.len());
    let mut min_tos = 0;
    let mut max_s = Vec::with_capacity(d.code.len());
    let mut max_tos = 0;
    for dc in d.code.iter() {
        match *dc {
            SudiceCode::Num(i) => {
                min_s.push(min_tos);
                min_tos = i;
                max_s.push(max_tos);
                max_tos = i;
            },
            SudiceCode::Roll => {
                let _ = min_s.pop().unwrap();
                let max_x = max_s.pop().unwrap();
                max_tos = max_x * max_tos;
            },
            SudiceCode::Add => {
                let min_x = min_s.pop().unwrap();
                min_tos = min_x + min_tos;
                let max_x = max_s.pop().unwrap();
                max_tos = max_x + max_tos;
            },
            SudiceCode::Sub => {
                let min_x = min_s.pop().unwrap();
                min_tos = min_tos - min_x;
                let max_x = max_s.pop().unwrap();
                max_tos = max_tos - max_x;
            },
            SudiceCode::Mul => {
                let min_x = min_s.pop().unwrap();
                min_tos = min_x * min_tos;
                let max_x = max_s.pop().unwrap();
                max_tos = max_x * max_tos;
            },
            SudiceCode::Div => {
                let min_x = min_s.pop().unwrap();
                min_tos = min_tos / min_x;
                let max_x = max_s.pop().unwrap();
                max_tos = max_tos / max_x;
            }
        }
    }
    (min_tos, max_tos)
}

pub fn compile(w: &mut Write, d: &Descriptor) {
    writeln!(w, "#include <random>");
    writeln!(w, "using namespace std;");
    writeln!(w, "uint64_t min = {};", min);
    writeln!(w, "uint64_t max = {};", max);
    writeln!(w, "uint64_t samples = {};", samples);
    writeln!(w, "uint64_t hist[{}];", size);
    writeln!(w, "int main() {");
    writeln!(w, "  std::default_random_engine gen;");
    writeln!(w, "  std::uniform_int_distribution<int64_t> dist({},{});");
    writeln!(w, "  return 0;");
    writeln!(w, "}");
}

#[derive(Debug)]
enum SudiceValue {
    Scalar(i64),
    Vector(i64, Vec<i64>)
}

trait HasSudiceValue {
    fn as_value(self) -> SudiceValue;
}

impl HasSudiceValue for i64 {
    fn as_value(self) -> SudiceValue {
        SudiceValue::Scalar(self)
    }
}

impl HasSudiceValue for SudiceValue {
    fn as_value(self) -> SudiceValue {
        self
    }
}

impl SudiceValue {
    fn new<T: HasSudiceValue>(value: T) -> SudiceValue {
        value.as_value()
    }

    fn collapse(self) -> i64 {
        match self {
            SudiceValue::Scalar(i) => i,
            SudiceValue::Vector(s) => s.iter().fold(0, |acc, &x| acc + x),
        }
    }

    fn add<T: HasSudiceValue>(self, value: T) -> SudiceValue {
        SudiceValue::Scalar(self.collapse() + value.as_value.collapse())
    }

    fn sub<T: HasSudiceValue>(self, value: T) -> SudiceValue {
        SudiceValue::Scalar(self.collapse() - value.as_value.collapse())
    }

    fn mul<T: HasSudiceValue>(self, value: T) -> SudiceValue {
        SudiceValue::Scalar(self.collapse() * value.as_value.collapse())
    }

    fn div<T: HasSudiceValue>(self, value: T) -> SudiceValue {
        SudiceValue::Scalar(self.collapse() / value.as_value.collapse())
    }

    fn roll<T: HasSudiceValue, S: HasSudiceValue>(num: T, size: S, r: &mut ThreadRng) -> SudiceValue {
        let n = num.as_value();
        let x = size.as_value();
        let btwn = Range::new(1, x+1);
        let v = Vec::with_capacity(n);
        for i in 0..n {
            v.push(btwn.ind_sample(r));
        }
        SudiceValue::Vector(x, v)
    }

    fn reroll<T: HasSudiceValue>(self, value: T, r: &mut ThreadRng) -> SudiceValue {
        let n = value.as_value().collapse();
        match self {
            SudiceValue::Scalar(_) => panic!("Error: Cannot reroll a scalar value."),
            SudiceValue::Vector(x, mut v) => {
                let btwn = Range::new(1, x+1);
                for i in 0..v.len() {
                    if v[i] == n {
                        v[i] = btwn.ind_sample(r);
                    }
                }
                self
            },
        }
    }
}

pub fn interpret(d: &SudiceExpression, r: &mut ThreadRng) -> i64 {
    let mut s = Vec::with_capacity(d.code.len());
    let mut tos = SudiceValue::Scalar(0);
    let op2 = |f| {
        let x = s.pop().unwrap();
        tos = f(tos, x);
    };
    let rop = |f| {
        let x = s.pop().unwrap();
        tos = f(tos, x, r);
    };
    for dcp in 0..d.code.len() {
        match d.code[dcp] {
            SudiceCode::Num(i) => {
                s.push(tos);
                tos = SudiceValue::new(i);
            },
            SudiceCode::Add => op2(SudiceValue::add),
            SudiceCode::Sub => op2(SudiceValue::sub),
            SudiceCode::Mul => op2(SudiceValue::mul),
            SudiceCode::Div => op2(SudiceValue::div),
            SudiceCode::Roll => rop(SudiceValue::roll),
            SudiceCode::Reroll => rop(SudiceValue::reroll),
        }
    }
    tos.collapse()
}
