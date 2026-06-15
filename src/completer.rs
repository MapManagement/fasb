use crate::config::*;
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};

pub const FACET_COMMANDS: &[&str] = &[
    // Primär
    ACTIVATE_FACETS, ACTIVATE_FACETS_LT, ACTIVATE_FACETS_LAZY,
    SHOW_FACETS, COMPUTE_FACETS_SU, COMPUTE_FACETS,
    FACET_COUNT, FACET_COUNTS, FACET_COUNTS_PROJECTING,
    WEIGHTED_FACET_COUNT, WEIGHTED_FACET_COUNTS,
    SHOW_ROUTE, CLEAR_ROUTE, DEL_LAST,
    SIGNIFICANCE, SIGNIFICANCE_PROJECTING,
    IS_FACET, IS_FACET_R,
    // Aliase
    ACTIVATE_FACETS_ALIAS, ACTIVATE_FACETS_LT_ALIAS, ACTIVATE_FACETS_LAZY_ALIAS,
    SHOW_FACETS_ALIAS, COMPUTE_FACETS_SU_ALIAS, COMPUTE_FACETS_ALIAS,
    FACET_COUNT_ALIAS, FACET_COUNTS_ALIAS, FACET_COUNTS_PROJECTING_ALIAS,
    WEIGHTED_FACET_COUNT_ALIAS, WEIGHTED_FACET_COUNTS_ALIAS,
    SHOW_ROUTE_ALIAS, CLEAR_ROUTE_ALIAS, DEL_LAST_ALIAS,
    SIGNIFICANCE_ALIAS, SIGNIFICANCE_PROJECTING_ALIAS,
    IS_FACET_ALIAS, IS_FACET_R_ALIAS,
];

pub const ATOM_COMMANDS: &[&str] = &[
    SHOW_ATOMS, FILTER_ATOMS, IS_ATOM, CONTEXT, ENTAILMENT, SOE,
    // Aliase
    CONTEXT_ALIAS,
];

pub const COMPARATOR: &[&str] = &[
    WHILE_LOOP_OP_GT,
    WHILE_LOOP_OP_GTE,
    WHILE_LOOP_OP_LT,
    WHILE_LOOP_OP_LTE,
    WHILE_LOOP_OP_NEQ,
];

pub const METRIC: &[&str] = &[WHILE_LOOP_VAR_FACETS, WHILE_LOOP_VAR_ROUTE];

pub const OTHER_COMMANDS: &[&str] = &[
    ANSWER_SET_COUNT, ANSWER_SET_COUNTS,
    ENUMERATE_SOLUTIONS, ENUMERATE_PROJECTED_SOLUTIONS,
    CHANGE_MODE, DISPLAY_MODE,
    PROPOSE_STEP, TAKE_STEP,
    SHOW_PROGRAM, QUIT, LOOP, MANUAL, COMPUTE_FACETS_SOE,
    // Aliase
    ANSWER_SET_COUNT_ALIAS, ANSWER_SET_COUNTS_ALIAS,
    ENUMERATE_SOLUTIONS_ALIAS, ENUMERATE_PROJECTED_SOLUTIONS_ALIAS,
    CHANGE_MODE_ALIAS, PROPOSE_STEP_ALIAS, TAKE_STEP_ALIAS,
];

enum Slot<'a> {
    Command,
    Names(&'a str),
    LoopNames(&'a str),
    Comparator,
    Metric,
    Nothing,
}

// Get the current partial word and start of word
fn current_word(line: &str, pos: usize) -> (usize, &str) {
    let word_start = line[..pos]
        .rfind(char::is_whitespace)
        .map(|i| i + 1)
        .unwrap_or(0);
    (word_start, &line[word_start..pos])
}

fn classify(line: &str, word_start: usize) -> Slot<'_> {
    let left = line[..word_start].trim_start();

    let Some(body) = left.strip_prefix(LOOP) else {
        return match left.split_whitespace().next() {
            None => Slot::Command,
            Some(cmd) => Slot::Names(cmd),
        };
    };

    let body = body.trim_start();

    if let Some((_cond, after_do)) = body.split_once(WHILE_LOOP_DO) {
        let current = after_do
            .rsplit(WHILE_LOOP_CMD_SEP)
            .next()
            .unwrap_or("")
            .trim_start();
        match current.split_whitespace().next() {
            None => Slot::Command,
            Some(cmd) => Slot::LoopNames(cmd),
        }
    } else {
        match body.split_whitespace().count() {
            0 => Slot::Comparator,
            1 => Slot::Metric,
            _ => Slot::Nothing,
        }
    }
}

pub struct FasbHelper {
    pub atoms: Vec<String>,
    pub facets: Vec<String>,
}

impl FasbHelper {
    pub fn new() -> Self {
        Self {
            atoms: vec![],
            facets: vec![],
        }
    }

    pub fn update(&mut self, atoms: &[String], facets: &[String]) {
        self.atoms = atoms.to_vec();
        self.facets = facets.to_vec();
    }

    fn candidates(&self, slot: Slot, word: &str) -> Vec<Pair> {
        match slot {
            Slot::Command => OTHER_COMMANDS
                .iter()
                .chain(FACET_COMMANDS.iter())
                .chain(ATOM_COMMANDS.iter())
                .filter(|c| c.starts_with(word))
                .map(|c| Pair {
                    display: c.to_string(),
                    replacement: c.to_string(),
                })
                .collect(),
            Slot::Names(cmd) | Slot::LoopNames(cmd) => {
                let in_loop = matches!(slot, Slot::LoopNames(_));

                let mut names: Vec<&String> = Vec::new();
                if ATOM_COMMANDS.contains(&cmd) {
                    names.extend(&self.atoms);
                } else if FACET_COMMANDS.contains(&cmd) {
                    names.extend(&self.facets);
                } else {
                    names.extend(&self.atoms);
                    names.extend(&self.facets);
                }

                let mut pairs: Vec<Pair> = names
                    .iter()
                    .filter(|c| c.starts_with(word))
                    .map(|c| Pair {
                        display: c.to_string(),
                        replacement: c.to_string(),
                    })
                    .collect();

                if in_loop && WHILE_LOOP_CMD_SEP.starts_with(word) {
                    pairs.push(Pair {
                        display: WHILE_LOOP_CMD_SEP.to_string(),
                        replacement: WHILE_LOOP_CMD_SEP.to_string(),
                    });
                }
                pairs
            }
            Slot::Comparator => COMPARATOR
                .iter()
                .filter(|c| c.starts_with(word))
                .map(|c| Pair {
                    display: c.to_string(),
                    replacement: c.to_string(),
                })
                .collect(),
            Slot::Metric => METRIC
                .iter()
                .filter(|c| c.starts_with(word))
                .map(|c| Pair {
                    display: c.to_string(),
                    replacement: c.to_string(),
                })
                .collect(),
            Slot::Nothing => vec![],
        }
    }
}

impl Completer for FasbHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let (word_start, word) = current_word(line, pos);
        let slot = classify(line, word_start);
        let candidates = self.candidates(slot, word);

        Ok((word_start, candidates))
    }
}

impl Hinter for FasbHelper {
    type Hint = String;
}
impl Highlighter for FasbHelper {}
impl Validator for FasbHelper {}
impl Helper for FasbHelper {}