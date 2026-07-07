mod completer;
mod config;
mod interpreter;
mod is_facet;
mod modes;
mod significance;
mod wfc;
mod wrappers;

#[pyo3::pymodule]
pub mod fasb {
    use crate::completer::FasbHelper;
    use crate::config;
    use crate::config::PROMPT;
    use crate::interpreter::interpreter_bindings::command;
    use crate::modes::Mode;
    use crate::wrappers::wrappers_bindings::PyNavigator;
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;
    use pyo3::pyfunction;
    use regex::Regex;
    use rustyline::error::ReadlineError;
    use rustyline::{CompletionType, Config, Editor, history::DefaultHistory};
    use savan::lex;
    use savan::nav::facets::Facets;
    use std::fs::read_to_string;
    use std::path::Path;

    #[pymodule_export]
    use crate::interpreter::interpreter_bindings;

    #[pymodule_export]
    use crate::wrappers::wrappers_bindings;

    #[pyfunction]
    pub fn start_fasb(
        args: Vec<String>,
        logic_programm_path: String,
        supress_facet_computation: bool,
        print_atoms: bool,
    ) -> PyResult<()> {
        let facets_at_startup = supress_facet_computation;
        let learned_that_at_startup = print_atoms;

        let lp = read_to_string(Path::new(&logic_programm_path))
            .map_err(|_| PyRuntimeError::new_err("read_to_string failed"))?;

        let re = Regex::new(r#config::FILTER_KEYWORD).unwrap();
        let filter_re = match lp.lines().last() {
            Some(x) => {
                if re.is_match(x) {
                    let s = &x.replace(config::FILTER_KEYWORD, "");
                    Regex::new(r#s).map_err(|_| PyRuntimeError::new_err("Regex Filter failed"))?
                } else {
                    let s = "";
                    Regex::new(r#s).map_err(|_| PyRuntimeError::new_err("Regex Filter failed"))?
                }
            }
            _ => todo!(),
        };

        println!(
            "{} v{} (repl)",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );

        let mut py_nav = PyNavigator::new(lp.clone(), args.clone())?;
        // Not needed anymore
        let mut _mode = Mode::GoalOriented(None::<usize>);
        let mut atoms = py_nav
            .nav
            .atoms()
            .filter(|a| filter_re.is_match(a))
            .collect::<Vec<String>>();
        let mut route = Vec::new();
        let mut cnf = Vec::new();

        let mut facets = if facets_at_startup {
            match learned_that_at_startup {
                false => py_nav
                    .nav
                    .facet_inducing_atoms(route.iter())
                    .ok_or(PyRuntimeError::new_err("nav.facet_inducing_atoms failed"))?
                    .into_iter()
                    .map(lex::repr)
                    .collect::<Vec<_>>(),
                _ => py_nav
                    .nav
                    .learned_that(&atoms, &route, None)
                    .ok_or(PyRuntimeError::new_err("nav.learned_that failed"))?,
            }
        } else {
            vec![]
        };

        let config = Config::builder()
            .completion_type(CompletionType::List)
            .build();
        let mut rl: Editor<FasbHelper, DefaultHistory> = Editor::with_config(config)
            .map_err(|_| PyRuntimeError::new_err("Editor::with_config() failed"))?;
        rl.set_helper(Some(FasbHelper::new()));

        for a in py_nav.nav.atoms() {
            if let Err(err) = rl.add_history_entry(a.as_str()) {
                eprintln!("ReadlineError: {:?}", err);
            }
        }

        loop {
            // Refresh completion candidates
            if let Some(helper) = rl.helper_mut() {
                helper.update(&atoms, &facets);
            }

            match rl.readline(crate::config::PROMPT) {
                Ok(line) => {
                    if let Err(err) = rl.add_history_entry(line.as_str()) {
                        eprintln!("ReadlineError: {:?}", err);
                    }

                    (atoms, facets, route, cnf) =
                        command(line, &mut py_nav, atoms, facets, route, cnf)?;
                }
                Err(ReadlineError::Interrupted) => {}
                Err(ReadlineError::Eof) => {
                    break;
                }
                Err(err) => {
                    eprintln!("ReadlineError: {:?}", err);
                }
            }
        }

        Ok(())
    }

    #[pyfunction]
    pub fn start_fasb_interpreter(
        args: Vec<String>,
        logic_programm_path: String,
        script_path: String,
        supress_facet_computation: bool,
        print_atoms: bool,
    ) -> PyResult<()> {
        println!(
            "{} v{} (interpreter)",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
        );

        let lp = read_to_string(Path::new(&logic_programm_path))
            .map_err(|_| PyRuntimeError::new_err("read_to_string for logic program failed"))?;
        let script = read_to_string(Path::new(&script_path))
            .map_err(|_| PyRuntimeError::new_err("read_to_string for script failed"))?;

        let re = Regex::new(r#config::FILTER_KEYWORD).unwrap();
        let filter_re = match lp.lines().last() {
            Some(x) => {
                if re.is_match(x) {
                    let s = &x.replace(config::FILTER_KEYWORD, "");
                    Regex::new(r#s).map_err(|_| PyRuntimeError::new_err("Regex Filter failed"))?
                } else {
                    let s = "";
                    Regex::new(r#s).map_err(|_| PyRuntimeError::new_err("Regex Filter failed"))?
                }
            }
            _ => todo!(),
        };

        let mut py_nav = PyNavigator::new(lp, args)?;
        // Not needed anymore
        let mut _mode = Mode::GoalOriented(None::<usize>);

        let mut atoms = py_nav
            .nav
            .atoms()
            .filter(|a| filter_re.is_match(a))
            .collect::<Vec<String>>();
        let mut route = Vec::new();
        let mut ctx = Vec::new();
        let mut facets = if supress_facet_computation {
            match print_atoms {
                false => py_nav
                    .nav
                    .facet_inducing_atoms(route.iter())
                    .ok_or(PyRuntimeError::new_err("nav.facet_inducing_atoms failed"))?
                    .iter()
                    .map(|f| lex::repr(*f))
                    .collect::<Vec<_>>(),
                _ => py_nav
                    .nav
                    .learned_that(&atoms, &route, None)
                    .ok_or(PyRuntimeError::new_err("nav.learned_that failed"))?,
            }
        } else {
            vec![]
        };

        for line in script.lines() {
            println!("{PROMPT}{line}");
            (atoms, facets, route, ctx) =
                command(line.to_owned(), &mut py_nav, atoms, facets, route, ctx)?
        }

        Ok(())
    }
}
