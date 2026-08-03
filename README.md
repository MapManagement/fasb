[![Crates.io](https://img.shields.io/crates/v/fasb?label=crates.io%20%28bin%29)](https://crates.io/crates/fasb)
![build workflow](https://github.com/drwadu/fasb/actions/workflows/build.yml/badge.svg)
![test workflow](https://github.com/drwadu/fasb/actions/workflows/test.yml/badge.svg)

# fasb

Implementation of the **f**aceted **a**nswer **s**et **b**rowser, introduced in https://doi.org/10.1609/aaai.v36i5.20506.

fasb is a REPL system implemented on top of the [clingo](https://github.com/potassco/clingo) solver.
It enables answer set navigation alongside quantitative reasoning.

## web tool

A prototypical web application of fasb can be found
[here](https://drwadu.github.io/web-fasb.github.io/). Its implementation is
very basic and not considered stable by any means. The compiled command line
tool is more efficient & provides more functionality.

## fundamental concepts

**weight of facet**

The weight of a facet is the amount by which a specified quantity changes due
to activating this facet. More on weights of facets can be found in
https://doi.org/10.1609/aaai.v36i5.20506.

**significance of a facet for a literal**

To ask how significant a facet `f` is for a literal `l`, conceptionally,
corresponds to asking how much information we gain (dually, uncertainty we
reduce) among answer sets that satisfy `l` when filtering those answer sets
that satisfy `l` and `f`. More on the notion of significance can be found in
https://doi.org/10.24963/kr.2024/60.

**representative answer sets**

fasb also implements a basic method for compressing a huge amount of answer
sets into representative ones. More on representative answer sets can be found
in https://ebooks.iospress.nl/doi/10.3233/FAIA230280.

## quickstart

fasb as a REPL:

```
$ fasb program.lp 0
fasb v0.1.2
:: ! 2         -- enumerate up to 2 answer sets
solution 1:
a e
solution 2:
b d e
found 2
:: ?           -- query facets
b d c a
:: #!!         -- query weights based on answer set counting
0.3333 2 b     -- [reduces answer set count by] [remaining answer sets] [facet]
0.6667 1 d
0.3333 2 ~d
0.6667 1 c
0.3333 2 ~c
0.6667 1 a
0.3333 2 ~a
:: ' max#f     -- use facet-counting strictly goal-oriented mode
:: $$          -- perform step (causing highest uncertainty reduction)
1.0000 0 d     -- activated facet `d` (reduced facet count by 100%)
:: @           -- query current route
d
:: !           -- enumerate all answer sets under current route
solution 1:
b d e
found 1
:: --          -- clear route
:: #!          -- query answer set count
3
:: > a|b&c|d   -- declare cnf query: (a or b) and (c or d)
:: >           -- clear query
:: % e ^*      -- compute significance of each current facet for literal e
 inc   exc
1.000 0.250 d
1.000 0.500 a
1.000 0.250 c
0.500 1.000 b
:: :q          --  quit
```

fasb as an interpreter:

```
$ cat script.fsb
! 1                  -- output 1 answer set
#?                   -- query facet count
\ != #f 0 | $$ . ! 2 -- while condition | command . command
@                    -- display route
$ fasb program.lp 0 srcipt.fsb
fasb v0.1.2
:: ! 1
solution 1:
a e
found 1
:: #?
8
:: \ != #f 0 | $$ . ! 2
_ _ b
solution 1:
b d e
solution 2:
b c e
found 2
_ _ c
solution 1:
b c e
found 1
:: @
b c
```

## install

1. [install cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
2. `cargo install fasb`

## build

1. [install cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
2. `cd fasb && cargo build -r`

## usage

`fasb program [clingo flags] [script]`

Apart from being a REPL system, fasb can also be used as an interpreter of
instructions, which will be performed line by line. To use fasb as an
interpreter add the feature flag `--feature interpreter` when installing or
building. When using the interpreter, provide a script.

The designated syntax for regular expressions (regex) can be found
[here](https://docs.rs/regex/latest/regex/).

### commands

Run fasb and type command `man` to see a palette of commands.

### parameters

- `--f` suppresses facet computation at startup
- `--l` prints true and false atoms at startup

## Python bindings

1. Install [maturin](https://github.com/pyo3/maturin)
2. Create new virtual environment for maturin: `python -m venv venv` and source it
   `source venv/bin/activate` (or use any package manager you prefer)
3. Build Python module using maturin: `maturin develop`
4. Start Python instance in venv and import fasb module: `import fasb`
5. Start fasb in Python: `fasb.start_fasb([], 'your_lp_file.lp', True, False)`

Currently you can use and import following (sub)modules:

```python
import fasb # lib.rs
from fasb import interpreter_bindings # interpreter.rs
from fasb import wrappers_bindings # wrappers.rs
```

## While Loop

| Constant Name | Current Command | Function | Old Symbol |
| --- | --- | --- | --- |
| WHILE_LOOP | `"while"` | Starts the loop | `"while"` |
| WHILE_LOOP_DO | `"do"` | Separates condition from command block | — |
| WHILE_LOOP_CMD_SEP | `";"` | Separates multiple commands | `"."` |
| WHILE_LOOP_VAR_FACETS | `"#facets"` | Variable: Current facet count | `"#f"` |
| WHILE_LOOP_VAR_ROUTE | `"#route"` | Variable: Current route length | `"#r"` |
| WHILE_LOOP_OP_NEQ | `"!="` | Not equal | `"!="` |
| WHILE_LOOP_OP_GT | `">"` | Greater than | `">"` |
| WHILE_LOOP_OP_GTE | `">="` | Greater than or equal to | `">="` |
| WHILE_LOOP_OP_LT | `"<"` | Less than | `"<"` |
| WHILE_LOOP_OP_LTE | `"<="` | Less than or equal to | `"<="` |

---

## Route Navigation

| Constant Name | Current Command | Function | Old Symbol |
| --- | --- | --- | --- |
| SHOW_ROUTE | `"route"` | Shows the current route | `"@"` |
| DEL_LAST | `"undo"` | Reverts the last facet (Undo) | `"-"` |
| CLEAR_ROUTE | `"clear"` | Clears the entire route | `"--"` |
| PROPOSE_STEP | `"propose"` | Proposes the next navigation step | `"$"` |
| TAKE_STEP | `"step"` | Executes the next step | `"$$"` |

---

## Facets

| Constant Name | Current Command | Function | Old Symbol |
| --- | --- | --- | --- |
| ACTIVATE_FACETS | `"activate"` (or `"+"`) | Activates specific facets | `"+"` |
| ACTIVATE_FACETS_LT | `"activate!"` | Activates facets and shows consequences | `"+'"` |
| ACTIVATE_FACETS_LAZY | `"lazy+"` | Lazy activation of facets | `":+"` |
| SHOW_FACETS | `"facets"` | Shows all currently induced facets | `"?"` |
| COMPUTE_FACETS | `"compute"` | Computes facets for specific targets | `"!?"` |
| COMPUTE_FACETS_SU | `"compute^"` | Computes facets (specific setup) | `"'!?"` |
| FACET_COUNT | `"count"` | Shows the current facet count | `"#?"` |
| FACET_COUNTS | `"counts"` | Shows counts under filtered facets | `"#??"` |
| FACET_COUNTS_PROJECTING | `"counts!"` | Shows counts using projection | `"!#??"` |
| WEIGHTED_FACET_COUNT | `"wcount"` | Weighted facet count | `"#?w"` |
| WEIGHTED_FACET_COUNTS | `"wcounts"` | Weighted counts under filter | `"#??w"` |
| IS_FACET | `"isfacet"` | Checks if atoms are facets | `":?"` |
| IS_FACET_R | `"isfacet!"` | Checks facets (with consistency check) | `":?r"` |

---

## Answer Sets

| Constant Name | Current Command | Function | Old Symbol |
| --- | --- | --- | --- |
| ENUMERATE_SOLUTIONS | `"solve"` | Outputs a specific number of answer sets | `"!"` |
| ENUMERATE_PROJECTED_SOLUTIONS | `"solve*"` | Outputs projected answer sets | `"!*"` |
| ANSWER_SET_COUNT | `"solvecount"` | Shows the total number of answer sets | `"#!"` |
| ANSWER_SET_COUNTS | `"solvecounts"` | Shows answer set counts under facets | `"#!!"` |

---

## System / Console

| Constant Name | Current Command | Function | Old Symbol |
| --- | --- | --- | --- |
| PROMPT | `":: "` | The command line prefix of the fASB console | `":: "` |
| SHOW_PROGRAM | `":src"` | Shows the source logic program | `":src"` |
| SHOW_ATOMS | `":atoms"` | Shows all atoms of the program | `":atoms"` |
| FILTER_ATOMS | `":filter_atoms"` | Filters atoms via regex | `":filter_atoms"` |
| IS_ATOM | `":isatom"` | Checks if a specific atom exists | `":isatom"` |
| CONTEXT | `":ctx"` | Declares a new logical context (CNF) | `">"` |
| SOE | `":soe"` | Counts/collects representative answer sets | `":soe"` |
| SIGNIFICANCE | `"sig"` | Shows significance of facets for a literal | `"%"` |
| SIGNIFICANCE_PROJECTING | `"sig*"` | Significance under projection | `"!%"` |
| ENTAILMENT | "\|=" | Logical entailment (cautious/brave) | "\|=" |
| DISPLAY_MODE | `":mode"` | Shows the navigation mode | `":mode"` |
| CHANGE_MODE | `":m"` | Changes the navigation mode | `"'"` |
| QUIT | `":q"` | Quits the interpreter | `":q"` |
| FILTER_KEYWORD | `"%filter "` | Keyword for internal filtering | `"%filter "` |

**Example (old):** `while #f | $$ . ?`
**Example (new):** `while #facets != 0 do step ; facets`
