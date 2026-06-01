use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};

// Commands from config.rs
pub const COMMANDS: &[&str] = &[
    "+", ":+", "+'", "?", "'!?", "!?", "#?", "#??", "!#??", "#?w", "#??w",
    "#!", "#!!", "!", "!*", "@", "--", "-", "'", ":mode", "$", "$$",
    ":src", ":atoms", ":filter_atoms", ":isatom", ":soe", ">", "%", "!%",
    ":?", ":?r", "|=", ":q", "man", "!?soe", "\\",
];

pub struct FasbHelper {
    pub atoms: Vec<String>,
    pub facets: Vec<String>,
}

impl FasbHelper {
    pub fn new() -> Self {
        Self { atoms: vec![], facets: vec![] }
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
            COMMANDS
                .iter()
                .filter(|c| c.starts_with(word))
                .map(|c| Pair { display: c.to_string(), replacement: c.to_string() })
                .collect()
        } else {
            // Negation prefix handeling
            let (prefix, stem) = match word.strip_prefix('~') {
                Some(rest) => ("~", rest),
                None => ("", word),
            };

            // Atoms first (the long ASP names), then facets, de-duplicated.
            let mut seen = std::collections::HashSet::new();
            self.atoms
                .iter()
                .chain(self.facets.iter())
                .filter(|name| name.starts_with(stem))
                // deduplication through seen set
                .filter(|name| seen.insert(name.as_str()))
                .map(|name| {
                    let full = format!("{prefix}{name}");
                    Pair { display: full.clone(), replacement: full }
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
