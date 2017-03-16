use descriptor::{SudiceCode, SudiceExpression};

use rand::distributions::{IndependentSample, Range};
use rand::ThreadRng;

use std::cmp;
use std::vec::Vec;

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
            SudiceValue::Vector(_, s) => s.iter().fold(0, |acc, &x| acc + x),
        }
    }

    fn add<T: HasSudiceValue>(self, value: T) -> SudiceValue {
        SudiceValue::Scalar(self.collapse() + value.as_value().collapse())
    }

    fn sub<T: HasSudiceValue>(self, value: T) -> SudiceValue {
        SudiceValue::Scalar(self.collapse() - value.as_value().collapse())
    }

    fn mul<T: HasSudiceValue>(self, value: T) -> SudiceValue {
        SudiceValue::Scalar(self.collapse() * value.as_value().collapse())
    }

    fn div<T: HasSudiceValue>(self, value: T) -> SudiceValue {
        SudiceValue::Scalar(self.collapse() / value.as_value().collapse())
    }

    fn roll<T: HasSudiceValue, S: HasSudiceValue>(num: T, size: S, r: &mut ThreadRng) -> SudiceValue {
        let n = num.as_value().collapse();
        let x = size.as_value().collapse();
        let btwn = Range::new(1, x+1);
        assert!(n > 0);
        let mut v = Vec::with_capacity(n as usize);
        for _ in 0..n {
            v.push(btwn.ind_sample(r));
        }
        v.sort();
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
                SudiceValue::Vector(x, v)
            },
        }
    }

    fn reroll_lowest<T: HasSudiceValue>(self, value: T, r: &mut ThreadRng) -> SudiceValue {
        let v = value.as_value().collapse();
        assert!(v >= 0);
        let n = v as usize;
        match self {
            SudiceValue::Scalar(_) => panic!("Error: Cannot reroll a scalar value."),
            SudiceValue::Vector(x, mut v) => {
                if n > v.len() {
                    panic!("Error: Cannot reroll {} from {} rolls.", n, v.len());
                }
                let btwn = Range::new(1, x+1);
                for i in 0..n {
                    v[i] = btwn.ind_sample(r);
                }
                v.sort();
                SudiceValue::Vector(x, v)
            },
        }
    }

    fn reroll_highest<T: HasSudiceValue>(self, value: T, r: &mut ThreadRng) -> SudiceValue {
        let v = value.as_value().collapse();
        assert!(v >= 0);
        let n = v as usize;
        match self {
            SudiceValue::Scalar(_) => panic!("Error: Cannot reroll a scalar value."),
            SudiceValue::Vector(x, mut v) => {
                if n > v.len() {
                    panic!("Error: Cannot reroll {} from {} rolls.", n, v.len());
                }
                let btwn = Range::new(1, x+1);
                for i in (v.len()-n)..v.len() {
                    v[i] = btwn.ind_sample(r);
                }
                v.sort();
                SudiceValue::Vector(x, v)
            },
        }
    }

    fn drop_lowest<T: HasSudiceValue>(self, value: T) -> SudiceValue {
        let v = value.as_value().collapse();
        assert!(v >= 0);
        let n = v as usize;
        match self {
            SudiceValue::Scalar(_) => panic!("Error: Cannot drop a scalar value."),
            SudiceValue::Vector(x, mut v) => {
                let len = v.len();
                if n >= len {
                    panic!("Error: Cannot drop {} from {} rolls.", n, v.len());
                }
                for i in 0..(len-n) {
                    v[i] = v[i+n];
                }
                v.truncate(len-n);
                SudiceValue::Vector(x, v)
            },
        }
    }

    fn drop_highest<T: HasSudiceValue>(self, value: T) -> SudiceValue {
        let v = value.as_value().collapse();
        assert!(v >= 0);
        let n = v as usize;
        match self {
            SudiceValue::Scalar(_) => panic!("Error: Cannot drop a scalar value."),
            SudiceValue::Vector(x, mut v) => {
                let len = v.len();
                if n >= len {
                    panic!("Error: Cannot drop {} from {} rolls.", n, v.len());
                }
                v.truncate(len-n);
                SudiceValue::Vector(x, v)
            },
        }
    }

    fn ceil<T: HasSudiceValue>(self, value: T) -> SudiceValue {
        let n = value.as_value().collapse();
        match self {
            SudiceValue::Scalar(s) => if s > n { SudiceValue::Scalar(n) } else { SudiceValue::Scalar(s) },
            SudiceValue::Vector(x, mut v) => {
                for i in 0..v.len() {
                    if v[i] > n {
                        v[i] = n;
                    }
                }
                SudiceValue::Vector(x, v)
            },
        }
    }

    fn floor<T: HasSudiceValue>(self, value: T) -> SudiceValue {
        let n = value.as_value().collapse();
        match self {
            SudiceValue::Scalar(s) => if s < n { SudiceValue::Scalar(n) } else { SudiceValue::Scalar(s) },
            SudiceValue::Vector(x, mut v) => {
                for i in 0..v.len() {
                    if v[i] < n {
                        v[i] = n;
                    }
                }
                SudiceValue::Vector(x, v)
            },
        }
    }

    fn into_bool(self) -> bool {
        self.collapse() == 1
    }

    fn from_bool(x: bool) -> SudiceValue {
        if x { SudiceValue::Scalar(1) } else { SudiceValue::Scalar(2) }
    }

    fn lt<T: HasSudiceValue>(self, value: T) -> SudiceValue {
        SudiceValue::from_bool(self.collapse() < value.as_value().collapse())
    }

    fn gt<T: HasSudiceValue>(self, value: T) -> SudiceValue {
        SudiceValue::from_bool(self.collapse() > value.as_value().collapse())
    }

    fn eq<T: HasSudiceValue>(self, value: T) -> SudiceValue {
        SudiceValue::from_bool(self.collapse() == value.as_value().collapse())
    }

    fn ne<T: HasSudiceValue>(self, value: T) -> SudiceValue {
        SudiceValue::from_bool(self.collapse() != value.as_value().collapse())
    }

    fn and<T: HasSudiceValue>(self, value: T) -> SudiceValue {
        SudiceValue::from_bool(self.into_bool() && value.as_value().into_bool())
    }

    fn or<T: HasSudiceValue>(self, value: T) -> SudiceValue {
        SudiceValue::from_bool(self.into_bool() || value.as_value().into_bool())
    }
}

struct Accumulator {
    pub ptr: usize,
    pub count: i64,
    pub value: i64
}

impl Accumulator {
    pub fn new(ptr: usize, count: i64, init: i64) -> Accumulator {
        Accumulator {
            ptr: ptr,
            count: count,
            value: init
        }
    }
}

pub fn interpret(d: &SudiceExpression, r: &mut ThreadRng) -> i64 {
    let mut l: Vec<Accumulator> = Vec::with_capacity(d.code.len());
    let mut s = Vec::with_capacity(d.code.len());
    let mut tos = SudiceValue::Scalar(0);
    let mut dcp = 0;
    macro_rules! op2 {
        ($func:path) => {{
            let x = s.pop().unwrap();
            tos = $func(tos, x);
        }}
    }

    macro_rules! rop {
        ($func:path) => {{
            let x = s.pop().unwrap();
            tos = $func(tos, x, r);
        }}
    }

    macro_rules! accum {
        ($func:path, $offset:ident) => {{
            let len = l.len();
            if len > 0 && l[len-1].ptr == dcp {
                if l[len-1].count <= 0 {
                    let a = l.pop().unwrap();
                    tos = SudiceValue::Scalar(a.value);
                } else {
                    l[len-1].value = $func(l[len-1].value, tos.collapse());
                    tos = s.pop().unwrap();
                    l[len-1].count -= 1;
                    dcp -= $offset + 1;
                }
            } else {
                let x = s.pop().unwrap().collapse();
                if x > 1 {
                    l.push(Accumulator::new(dcp, x - 1, tos.collapse()));
                    tos = s.pop().unwrap();
                    dcp -= $offset + 1;
                }
            }
        }}
    }
    while dcp < d.code.len() {
        match d.code[dcp] {
            SudiceCode::Num(i) => {
                s.push(tos);
                tos = SudiceValue::new(i);
            },
            SudiceCode::Add => op2!(SudiceValue::add),
            SudiceCode::Sub => op2!(SudiceValue::sub),
            SudiceCode::Mul => op2!(SudiceValue::mul),
            SudiceCode::Div => op2!(SudiceValue::div),
            SudiceCode::Roll => rop!(SudiceValue::roll),
            SudiceCode::Reroll => rop!(SudiceValue::reroll),
            SudiceCode::RerollLowest => rop!(SudiceValue::reroll_lowest),
            SudiceCode::RerollHighest => rop!(SudiceValue::reroll_highest),
            SudiceCode::DropLowest => op2!(SudiceValue::drop_lowest),
            SudiceCode::DropHighest => op2!(SudiceValue::drop_highest),
            SudiceCode::Ceil => op2!(SudiceValue::ceil),
            SudiceCode::Floor => op2!(SudiceValue::floor),
            SudiceCode::BestOf(offset) => accum!(cmp::max, offset),
            SudiceCode::WorstOf(offset) => accum!(cmp::min, offset),
            SudiceCode::Select(ref offsets) => {
                let t = tos.collapse();
                let x = t - 2;
                if x >= 0 && x < (offsets.len() as i64) {
                    dcp += offsets[x as usize];
                    tos = s.pop().unwrap();
                } else if t == 1 {
                    tos = s.pop().unwrap();
                } else {
                    dcp += *offsets.last().unwrap();
                    tos = SudiceValue::Scalar(t);
                }
            },
            SudiceCode::Jump(offset) => dcp += offset,
            SudiceCode::Lt => op2!(SudiceValue::lt),
            SudiceCode::Gt => op2!(SudiceValue::gt),
            SudiceCode::Eq => op2!(SudiceValue::eq),
            SudiceCode::Ne => op2!(SudiceValue::ne),
            SudiceCode::And => op2!(SudiceValue::and),
            SudiceCode::Or => op2!(SudiceValue::or),
        }
        dcp += 1;
    }
    tos.collapse()
}
