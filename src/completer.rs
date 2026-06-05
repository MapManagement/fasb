use crate::config::*;
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};

// Commands from config.rs
pub const FACET_COMMANDS: &[&str] = &[
    ACTIVATE_FACETS,
    ACTIVATE_FACETS_LAZY,
    ACTIVATE_FACETS_LT,
    SHOW_FACETS,
    COMPUTE_FACETS_SU,
    COMPUTE_FACETS,
    FACET_COUNT,
    FACET_COUNTS,
    FACET_COUNTS_PROJECTING,
    WEIGHTED_FACET_COUNT,
    WEIGHTED_FACET_COUNTS,
    SHOW_ROUTE,
    CLEAR_ROUTE,
    DEL_LAST,
    SIGNIFICANCE,
    SIGNIFICANCE_PROJECTING,
    IS_FACET,
    IS_FACET_R,
];

pub const ATOM_COMMANDS: &[&str] = &[SHOW_ATOMS, FILTER_ATOMS, IS_ATOM, CONTEXT, ENTAILMENT, SOE];

pub const COMPARATOR: &[&str] = &[BIGGER, BIGGEREQ, SMALLER, SMALLEREQ, NEQUAL];

pub const METRIC: &[&str] = &[FCOUNT, RLENGTH];

pub const OTHER_COMMANDS: &[&str] = &[
    ANSWER_SET_COUNT,
    ANSWER_SET_COUNTS,
    ENUMERATE_SOLUTIONS,
    ENUMERATE_PROJECTED_SOLUTIONS,
    CHANGE_MODE,
    DISPLAY_MODE,
    PROPOSE_STEP,
    TAKE_STEP,
    SHOW_PROGRAM,
    QUIT,
    LOOP,
    MANUAL,
    COMPUTE_FACETS_SOE,
];

enum Slot<'a> {
    Command,
    Names(&'a str),
    LoopNames(&'a str),
    Comparator,
    Metric,
    LoopSep,
    Nothing,
}

// Get the current partial word and start of word
fn current_word(line: &str, pos: usize) -> (usize, &str) {
    // word_start is the line until cursor, find the next char to last whitespace 
    let word_start = line[..pos]
        .rfind(char::is_whitespace)
        .map(|i| i + 1)
        .unwrap_or(0);
    (word_start, &line[word_start..pos])
}

// return Slots based on partial word
fn classify(line: &str, word_start: usize) -> Slot<'_> {
    // Context of line
    let left = line[..word_start].trim_start();

    // 
    let Some(body) = left.strip_prefix(LOOP) else {
        return match left.split_whitespace().next() {
            // Nothing -> Command
            None => Slot::Command,
            // Command -> Atom / Facet based on cmd
            Some(cmd) => Slot::Names(cmd),
        };
    };

    let body = body.trim_start();

    // Positions of predicates in Loop
    match body.split_once(LOOP_SEP) {
        None => match body.split_whitespace().count() {
            0 => Slot::Comparator,
            1 => Slot::Metric,
            2 => Slot::Nothing,
            _ => Slot::LoopSep,
        },

        // Predicate Logic
        Some((_predicate, instrs)) => {
            let current = instrs.rsplit(LOOP_END).next().unwrap_or("").trim_start();
            match current.split_whitespace().next() {
                None => Slot::Command,
                // Loop logic
                Some(cmd) => Slot::LoopNames(cmd),
            }
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
            // All commands suggested
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

                // Deduplication (through extend) + sugg logic
                let mut names: Vec<&String> = Vec::new();
                if ATOM_COMMANDS.contains(&cmd) {
                    names.extend(&self.atoms);
                } else if FACET_COMMANDS.contains(&cmd) {
                    names.extend(&self.facets);
                } else {
                    names.extend(&self.atoms);
                    names.extend(&self.facets);
                }

                // Search for matches for partial word
                let mut pairs: Vec<Pair> = names
                    .iter()
                    .filter(|c| c.starts_with(word))
                    .map(|c| Pair {
                        display: c.to_string(),
                        replacement: c.to_string(),
                    })
                    .collect();
                
                // Suggest . in list of instructions
                if in_loop && LOOP_END.starts_with(word) {
                    pairs.push(Pair {
                        display: LOOP_END.to_string(),
                        replacement: LOOP_END.to_string(),
                    });
                }
                pairs
            }
            // Predicate logic
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
            Slot::LoopSep => [LOOP_SEP]
                .iter()
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
        // Get partial word and start of word
        let (word_start, word) = current_word(line, pos);
        // Get Slot / suggestion category
        let slot = classify(line, word_start);
        // Get cantidates
        let candidates = self.candidates(slot, word);

        Ok((word_start, candidates))
    }
}

// Unused traits but required by rustyline
impl Hinter for FasbHelper {
    type Hint = String;
}
impl Highlighter for FasbHelper {}
impl Validator for FasbHelper {}
impl Helper for FasbHelper {}
