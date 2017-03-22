# Sudice

Sudice is an expression language for expressing probability distributions in
terms of dice rolls. It uses a sampling-based inference approach to approximate
the desired distribution.

## Getting Started

Compile with `cargo build --release` and then run `target/release/sudice` to 
start a REPL for the Sudice expression language.

### Tutorial

Sudice is an expression language, which basically means all programs are single
expressions. The simplest such expression is just arithmetic, such as

```
3 + 4
3 - 4
3 * 4
3 / 4
```

which each compute the appropriate distribution where one number occurs 100% of
the time. Note only integers are supported, so division will always round down.

Arguably the most important operator in the language is the dice roll operator.
For example,

```
2d20
```

would roll two twenty-sided dice. Naturally since the `d` operator is just
another operator, we can write expressions such as

```
2d3 - 6
1d2 * 10 + 1d6
```

We can also group expressions using parentheses like so

```
(3 + 4d2) * 10
```

We can even do some crazy stuff like conditioning the number of dice we roll
based off of another dice roll

```
(3d6)d2
3d(1d20)
```

Note that all the operators shown so far are left-associative within their
precedences, with `d` taking a higher precedence than other operations. So

```
3d6d2
```

is the same as

```
(3d6)d2
```

We also support dice rerolling such as

```
1d20rr1
1d20rl3
1d20rh15
```

which correspond to "reroll 1s", "reroll less than 3" and "reroll greater
than 15," respectively.

One can also drop dice from a dice roll by using

```
3d20\l1
3d20\h1
```

which correspond to "drop lowest 1 dice" and "drop highest 1 dice"
respectively. Note that the above operations only work on dice rolls.

In Sudice, there are two kinds of values: dice rolls and integers. Note
that the operator `d` expects two integers, but naturally `3d5` produces
3 dice rolls. So, how does `3d3d3` work? Basically, dice rolls are
implicitly converted into integers by summing them in places that expect
an integer. Note then, that

```
(3d20+2)\l1
```

would attempt to perform a drop operation on an integer! Luckily, Sudice
runs a semantic check before execution that ensures situations like this
will not happen. It will also stop one from dropping more dice than there
are in a single roll. If you wanted to still express this distribution,
there's usually a way around it. For the example above, one may simply write

```
3d20\l1 + 2
```

to express the same distribution.

Other neat operations in Sudice include `b` and `w` which correspond to
"best of" and "worst of" respectively. Essentially, they'll re-run any
expression some number of times, and pick either the best or the worst. For
example

```
1d20b2
```

is an easy way of expressing advantage in games like Dungeons & Dragons. That
is, "roll 2, take the highest one." However, this operation works on arbitrary
expressions, so you could do something like

```
(3d3d3 * 4 + 1d2 * 4)w3
```

which would run that inner expression three times and take the worst result.

One final key feature of Sudice is the select expression. It is a generalization
of a conditional statement which is useful for encoding piecewise probability
distributions. For example

```
[1d2 ? 1d20w2 : 1d20b2+10]
```

randomly selects either the first or second expression to run. Essentially, the
result of the first expression is used as an index into the rest of the
expressions. Any expression after the colon (:), however, is a "catch-all" 
expression and is always required. Thus, you could write something like

```
[1d3 ? 1d2 1d4 : 1d8]
```

Note that with these semantics, one can think of "true" as being 1 and "false"
as everything else. To support this notion, and a more natural form of
conditionals, we provide a set of conditional operators which return 1 if true 
and 2 as false.

```
1d20 > 8
1d100 < 24
```

Such behavior is useful for encoding something like a biased coin flip. One can
also say

```
1d20 = 1
1d20 <> 1
```

to encode equality and inequality. Finally, two boolean operators are provided in
the form of `and` and `or` which assume the value 1 to be true, and everything
else to be false.
