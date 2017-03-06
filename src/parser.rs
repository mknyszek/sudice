use descriptor::{SudiceCode, SudiceExpression};

use pest::prelude::*;

use std::collections::LinkedList;
use std::iter::FromIterator;

impl_rdp! {
    grammar! {
        expr = _{
            { ["("] ~ expr ~ [")"] | num }
            sum  = { plus  | minus }
            prod = { times | slash }
            dice = { roll | reroll | rerolll | rerollh | dropl | droph | ceil | floor | best | worst }
        }

        plus    =  { ["+"] }
        minus   =  { ["-"] }
        times   =  { ["*"] }
        slash   =  { ["/"] }
        roll    =  { ["d"] }
        reroll  =  { ["rr"] }
        rerolll =  { ["rrl"] }
        rerollh =  { ["rrh"] }
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
            (_: sum, mut left: _expr(), sign, mut right: _expr()) => {
                left.append(&mut right);
                left.push_front(match sign.rule {
                    Rule::plus  => SudiceCode::Add,
                    Rule::minus => SudiceCode::Sub,
                    _ => unreachable!()
                });
                left
            },
            (_: prod, mut left: _expr(), sign, mut right: _expr()) => {
                left.append(&mut right);
                left.push_front(match sign.rule {
                    Rule::times => SudiceCode::Mul,
                    Rule::slash => SudiceCode::Div,
                    _ => unreachable!()
                });
                left
            },
            (_: dice, mut left: _expr(), cmd, mut right: _expr()) => {
                let offset = left.len();
                left.append(&mut right);
                left.push_front(match cmd.rule {
                    Rule::roll    => SudiceCode::Roll,
                    Rule::reroll  => SudiceCode::Reroll,
                    Rule::rerolll => SudiceCode::RerollLowest,
                    Rule::rerollh => SudiceCode::RerollHighest,
                    Rule::dropl   => SudiceCode::DropLowest,
                    Rule::droph   => SudiceCode::DropHighest,
                    Rule::ceil    => SudiceCode::Ceil,
                    Rule::floor   => SudiceCode::Floor,
                    Rule::best    => SudiceCode::BestOf(offset),
                    Rule::worst   => SudiceCode::WorstOf(offset),
                    _ => unreachable!()
                });
                left
            }
        }
    }
}
