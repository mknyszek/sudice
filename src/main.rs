#[macro_use]
extern crate pest;
extern crate rand;

mod descriptor;
mod interpreter;

use descriptor::*;
use interpreter::*;

use std::collections::LinkedList;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::vec::Vec;

use pest::prelude::*;

impl_rdp! {
    grammar! {
        expr = _{
            { ["("] ~ expr ~ [")"] | num }
            sum  = { plus  | minus }
            prod = { times | slash }
            dice = { roll | reroll | rerolll | rerollh | drop | dropl | droph | ceil | floor | best | worst }
        }

        plus    =  { ["+"] }
        minus   =  { ["-"] }
        times   =  { ["*"] }
        slash   =  { ["/"] }
        roll    =  { ["d"] }
        reroll  =  { ["rr"] }
        rerolll =  { ["rrl"] }
        rerollh =  { ["rrh"] }
        drop    =  { ["\\"] }
        dropl   =  { ["\\l"] }
        droph   =  { ["\\h"] }
        ceil    =  { ["^"] }
        floor   =  { ["_"] }
        best    =  { ["b"] }
        worst   =  { ["w"] }

        num        = @{ ["0"] | ['1'..'9'] ~ ['0'..'9']* }
        whitespace = _{ [" "] }
    }

    process! {
        compile(&self) -> SudiceExpression {
            (expr: _expr()) => {
                let mut code = Vec::from_iter(expr.into_iter());
                code.reverse();
                SudiceExpression {
                    code: code,
                }
            }
        }
        _expr(&self) -> LinkedList<SudiceCode> {
            (&num: num) => {
                let mut dl = LinkedList::new();
                dl.push_front(SudiceCode::Num(num.parse::<i64>().unwrap()));
                dl
            },
            (_: sum, mut left: _dexpr(), sign, mut right: _dexpr()) => {
                left.append(&mut right);
                left.push_front(match sign.rule {
                    Rule::plus  => SudiceCode::Add,
                    Rule::minus => SudiceCode::Sub,
                    _ => unreachable!()
                });
                left
            },
            (_: prod, mut left: _dexpr(), sign, mut right: _dexpr()) => {
                left.append(&mut right);
                left.push_front(match sign.rule {
                    Rule::times => SudiceCode::Mul,
                    Rule::slash => SudiceCode::Div,
                    _ => unreachable!()
                });
                left
            },
            (_: dice, mut left: _dexpr(), cmd, mut right: _dexpr()) => {
                left.append(&mut right);
                left.push_front(match cmd.rule {
                    Rule::roll    => SudiceCode::Roll,
                    Rule::reroll  => SudiceCode::Reroll,
                    Rule::rerolll => SudiceCode::RerollLowest,
                    Rule::rerollh => SudiceCode::RerollHighest,
                    Rule::drop    => SudiceCode::Drop,
                    Rule::dropl   => SudiceCode::DropLowest,
                    Rule::droph   => SudiceCode::DropHighest,
                    Rule::ceil    => SudiceCode::Ceil,
                    Rule::floor   => SudiceCode::Floor,
                    Rule::best    => SudiceCode::BestOf,
                    Rule::worst   => SudiceCode::WorstOf,
                    _ => unreachable!()
                });
                left
            }
        }
    }
}

fn main() {
    let mut parser = Rdp::new(StringInput::new("(2d8 - 3)_0"));
    assert!(parser.expr());
    let mut rng = rand::thread_rng();
    let code = parser.compile();
    let (min, max) = compute_range(&code);
    let size = (max - min + 1) as usize;
    let mut hist: Vec<u64> = Vec::with_capacity(size);
    hist.resize(size, 0);
    for _ in 0..10000 {
        let s = interpret(&code, &mut rng);
        hist[(s - min) as usize] += 1;
    }
    println!("{:?}", hist);
}
