use descriptor::{SudiceCode, SudiceExpression};

use pest::prelude::*;

use std::collections::LinkedList;
use std::iter::FromIterator;

impl_rdp! {
    grammar! {
        expr = _{
            { ["("] ~ expr ~ [")"] | select | num }
            sum  = { plus  | minus }
            prod = { times | slash }
            dice = { roll | reroll | rerolll | rerollh | dropl | droph | ceil | floor | best | worst }
        }
        select  =  { bleft ~ expr ~ qmark ~ expr+ ~ bright }

        plus    =  { ["+"] }
        minus   =  { ["-"] }
        times   =  { ["*"] }
        slash   =  { ["/"] }
        roll    =  { ["d"] }
        reroll  =  { ["rr"] }
        rerolll =  { ["rl"] }
        rerollh =  { ["rh"] }
        dropl   =  { ["\\l"] }
        droph   =  { ["\\h"] }
        ceil    =  { ["^"] }
        floor   =  { ["_"] }
        best    =  { ["b"] }
        worst   =  { ["w"] }
        qmark   =  { ["?"] }
        bleft   =  { ["["] }
        bright  =  { ["]"] }

        num        = @{ ["0"] | ['1'..'9'] ~ ['0'..'9']* }
        whitespace = _{ [" "] | ["\n"] | ["\r"] }
    }

    process! {
        compile(&self) -> SudiceExpression {
            (expr: _expr()) => {
                let code = Vec::from_iter(expr.into_iter());
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
                right.append(&mut left);
                right.push_back(match sign.rule {
                    Rule::plus  => SudiceCode::Add,
                    Rule::minus => SudiceCode::Sub,
                    _ => unreachable!()
                });
                right
            },
            (_: prod, mut left: _expr(), sign, mut right: _expr()) => {
                right.append(&mut left);
                right.push_back(match sign.rule {
                    Rule::times => SudiceCode::Mul,
                    Rule::slash => SudiceCode::Div,
                    _ => unreachable!()
                });
                right
            },
            (_: dice, mut left: _expr(), cmd, mut right: _expr()) => {
                let offset = left.len();
                right.append(&mut left);
                right.push_back(match cmd.rule {
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
                right
            },
            (_: select, _: bleft, mut pred: _expr(), _: qmark, mut rest: _jump_seq()) => {
                let mut sum: usize = 0;
                rest.1.reverse();
                for i in 0..rest.1.len() {
                    sum += rest.1[i];
                    rest.1[i] = sum;
                }
                pred.push_back(SudiceCode::Select(rest.1));
                pred.append(&mut rest.0);
                pred
            }
        }
        _jump_seq(&self) -> (LinkedList<SudiceCode>, Vec<usize>) {
            (_: bright) => {
                (LinkedList::new(), Vec::new())
            },
            (mut head: _expr(), mut rest: _jump_seq()) => {
                let offset = rest.0.len();
                if offset != 0 {
                    head.push_back(SudiceCode::Jump(offset));
                }
                rest.1.push(head.len());
                head.append(&mut rest.0);
                (head, rest.1) 
            }
        }
    }
}
