use descriptor::{SudiceCode, SudiceExpression};

use pest::prelude::*;

use std::collections::LinkedList;
use std::iter::FromIterator;

impl_rdp! {
    grammar! {
        expr = _{
            { ["("] ~ expr ~ [")"] | select | num }
            bnry = { and | or }
            cond = { lt | gt | eq | ne }
            sum  = { plus  | minus }
            prod = { times | slash }
            dice = { roll | reroll | rerolll | rerollh | dropl | droph | ceil | floor | best | worst }
        }
        select =  { selbegin ~ expr ~ qmark ~ expr+ ~ ecase ~ expr ~ selend }

        plus     =  { ["+"] }
        minus    =  { ["-"] }
        times    =  { ["*"] }
        slash    =  { ["/"] }
        roll     =  { ["d"] }
        reroll   =  { ["rr"] }
        rerolll  =  { ["rl"] }
        rerollh  =  { ["rh"] }
        dropl    =  { ["\\l"] }
        droph    =  { ["\\h"] }
        ceil     =  { ["^"] }
        floor    =  { ["_"] }
        best     =  { ["b"] }
        worst    =  { ["w"] }
        qmark    =  { ["?"] }
        ecase    =  { [":"] }
        selbegin =  { ["["] }
        selend   =  { ["]"] }
        lt       =  { ["<"] }
        gt       =  { [">"] }
        eq       =  { ["=="] }
        ne       =  { ["!="] }
        and      =  { ["and"] }
        or       =  { ["or"] }

        num        = @{ ["-"]? ~ (["0"] | ['1'..'9'] ~ ['0'..'9']*) }
        whitespace = _{ [" "] }
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
            (_: bnry, mut left: _expr(), op, mut right: _expr()) => {
                right.append(&mut left);
                right.push_back(match op.rule {
                    Rule::and  => SudiceCode::And,
                    Rule::or => SudiceCode::Or,
                    _ => unreachable!()
                });
                right
            },
            (_: cond, mut left: _expr(), sign, mut right: _expr()) => {
                right.append(&mut left);
                right.push_back(match sign.rule {
                    Rule::lt => SudiceCode::Lt,
                    Rule::gt => SudiceCode::Gt,
                    Rule::eq => SudiceCode::Eq,
                    Rule::ne => SudiceCode::Ne,
                    _ => unreachable!()
                });
                right
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
            (_: select, _: selbegin, mut pred: _expr(), _: qmark, mut rest: _jump_seq()) => {
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
            (_: ecase, mut end: _expr(), _: selend) => {
                end.push_back(SudiceCode::Jump(0));
                let v = vec![end.len()];
                (end, v)
            },
            (mut head: _expr(), mut rest: _jump_seq()) => {
                let offset = rest.0.len();
                head.push_back(SudiceCode::Jump(offset));
                rest.1.push(head.len());
                head.append(&mut rest.0);
                (head, rest.1) 
            }
        }
    }
}
