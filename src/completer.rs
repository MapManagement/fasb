use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use crate::config::*;

// Commands from config.rs
pub const FACET_COMMANDS: &[&str] = &[
    ACTIVATE_FACETS, ACTIVATE_FACETS_LAZY, ACTIVATE_FACETS_LT,
    SHOW_FACETS, COMPUTE_FACETS_SU, COMPUTE_FACETS,
    FACET_COUNT, FACET_COUNTS, FACET_COUNTS_PROJECTING,
    WEIGHTED_FACET_COUNT, WEIGHTED_FACET_COUNTS,
    SHOW_ROUTE, CLEAR_ROUTE, DEL_LAST,
    SIGNIFICANCE, SIGNIFICANCE_PROJECTING,
    IS_FACET, IS_FACET_R,
];

pub const ATOM_COMMANDS: &[&str] = &[
    SHOW_ATOMS, FILTER_ATOMS, IS_ATOM,
    CONTEXT, ENTAILMENT, SOE,
];

pub const OTHER_COMMANDS: &[&str] = &[
    ANSWER_SET_COUNT, ANSWER_SET_COUNTS,
    ENUMERATE_SOLUTIONS, ENUMERATE_PROJECTED_SOLUTIONS,
    CHANGE_MODE, DISPLAY_MODE,
    PROPOSE_STEP, TAKE_STEP,
    SHOW_PROGRAM, QUIT,
];

pub struct FasbHelper {
    pub atoms: Vec<String>,
    pub facets: Vec<String>,
}

impl FasbHelper {
    pub fn new() -> Self {
        Self { atoms: vec![], facets: vec![]}
    }

    // Update FasbHelper
    pub fn update(&mut self, atoms: &[String], facets: &[String]) {
        self.atoms = atoms.to_vec();
        self.facets = facets.to_vec();
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
        // Start of the word under the cursor
        let word_start = line[..pos]
            .rfind(char::is_whitespace)
            .map(|i| i + 1)
            .unwrap_or(0);
        let word = &line[word_start..pos];

        // Bool if its the first token
        let is_first_token = line[..word_start].trim().is_empty();

        let candidates: Vec<Pair> = if is_first_token {
            // Case first token -> complete commands
            FACET_COMMANDS
                .iter()
                .chain(ATOM_COMMANDS)
                .chain(OTHER_COMMANDS)
                .filter(|c| c.starts_with(word))
                .map(|c| Pair { display: c.to_string(), replacement: c.to_string() })
                .collect()
        } else {
            let first_word = line[..word_start]
                .split_whitespace()
                .next()
                .unwrap_or("");

            let use_atoms  = ATOM_COMMANDS.contains(&first_word);
            let use_facets = FACET_COMMANDS.contains(&first_word);

            // Negation prefix handeling
            let (prefix, stem) = match word.strip_prefix('~') {
                Some(rest) => ("~", rest),
                None => ("", word),
            };

            // Atoms first (the long ASP names), then facets, de-duplicated.
            let mut seen = std::collections::HashSet::new();

            let mut names: Vec<&String> = Vec::new();
            if use_atoms  { names.extend(self.atoms.iter()); }
            else if use_facets { names.extend(self.facets.iter()); }
            else { names.extend(self.atoms.iter()); names.extend(self.facets.iter()); }

            names.into_iter()
            .filter(|name| name.starts_with(stem) && seen.insert(*name))
            .map(|name| Pair {
                display:     format!("{prefix}{name}"),
                replacement: format!("{prefix}{name}"),
            })
            .collect()
        };

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
