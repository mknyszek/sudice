use std::vec::Vec;

#[derive(Debug)]
pub enum SudiceCode {
    Num(i64),
    Add,
    Sub,
    Mul,
    Div,
    Roll,
    Reroll,
    RerollLowest,
    RerollHighest,
    DropLowest,
    DropHighest,
    Ceil,
    Floor,
    BestOf(usize),
    WorstOf(usize),
    Select(Vec<usize>),
    Jump(usize),
    Lt,
    Gt,
    Eq,
    Ne,
    And,
    Or,
    Abs,
    Neg
}

#[derive(Debug)]
pub struct SudiceExpression {
    pub code: Vec<SudiceCode>,
}
