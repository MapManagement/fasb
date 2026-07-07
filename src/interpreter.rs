#[pyo3::pymodule]
pub mod interpreter_bindings {
    use crate::cache::{cached_facets, cached_facets_projecting};
    use crate::config::*;
    use crate::is_facet;
    use crate::modes::Mode;
    use crate::significance::Significance;
    use crate::wfc::parse_weighted_facets_from_file;
    use crate::wfc::weighted_facet_count;
    use crate::wrappers::wrappers_bindings::ModeOptionUsize;
    use crate::wrappers::wrappers_bindings::perform_next_step_option_usize;
    use crate::wrappers::wrappers_bindings::{PyNavigator, propose_next_step_option_usize};
    use indicatif::{ProgressBar, ProgressState, ProgressStyle};
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;
    use regex::Regex;
    use savan::lex;
    use savan::nav::{
        facets::Facets,
        soe::Collect,
        weights::{Weight, count, count_projecting},
    };
    use std::fmt::Write;
    use std::num::NonZeroUsize;
    use std::thread;
    use std::time::Duration;
    use std::time::Instant;

    #[pyfunction]
    pub fn activate_facets(
        nav: &mut PyNavigator,
        mut route: Vec<String>,
        args: Vec<String>,
    ) -> PyResult<(Vec<String>, Vec<String>)> {
        route.extend(args);
        let facets = cached_facets(nav, &route)?;

        Ok((facets, route))
    }

    #[pyfunction]
    pub fn activate_facets_lt(
        nav: &mut PyNavigator,
        mut facets: Vec<String>,
        mut route: Vec<String>,
        args: Vec<String>,
    ) -> PyResult<(Vec<String>, Vec<String>)> {
        route.extend(args);

        facets = nav
            .nav
            .learned_that(&facets, &route, None)
            .ok_or(PyRuntimeError::new_err("activate_facets_lt failed"))?;

        Ok((facets, route))
    }

    #[pyfunction]
    pub fn activate_facets_lazy(
        mut route: Vec<String>,
        args: Vec<String>,
    ) -> PyResult<Vec<String>> {
        route.extend(args);
        Ok(route)
    }

    #[pyfunction]
    pub fn compute_facets(
        nav: &mut PyNavigator,
        route: Vec<String>,
        args: Vec<String>,
    ) -> PyResult<Vec<String>> {
        let start = Instant::now();
        let all_facets = cached_facets(nav, &route)?;
        let facets = if let Some(re) = args.first().and_then(|s| Regex::new(r#s).ok()) {
            all_facets
                .into_iter()
                .filter(|a| re.is_match(a))
                .collect::<Vec<_>>()
        } else {
            all_facets
        };
        println!("time elapsed: {:?}", start.elapsed());
        Ok(facets)
    }

    #[pyfunction]
    pub fn entailment(
        nav: &mut PyNavigator,
        atoms: Vec<String>,
        route: Vec<String>,
        args: Vec<String>,
    ) -> PyResult<()> {
        let start = Instant::now();

        let fst = args.first().map(|s| s.as_str());

        match fst {
            Some("%") => {
                if let Some(xs) = nav
                    .nav
                    .cautious_consequences(route.iter())
                    .map(|fs| fs.iter().map(|f| lex::repr(*f)).collect::<Vec<_>>())
                {
                    if let Some(re) = args.get(1).and_then(|s| Regex::new(r#s).ok()) {
                        for f in atoms.iter() {
                            if re.is_match(f) && xs.contains(f) {
                                println!("{f}");
                            }
                        }
                    } else {
                        for f in atoms.iter() {
                            if xs.contains(f) {
                                println!("{f}");
                            }
                        }
                    }
                }
            }

            Some("%%") => {
                if let Some(xs) = nav
                    .nav
                    .brave_consequences(route.iter())
                    .map(|fs| fs.iter().map(|f| lex::repr(*f)).collect::<Vec<_>>())
                {
                    if let Some(re) = args.get(1).and_then(|s| Regex::new(r#s).ok()) {
                        for f in atoms.iter() {
                            if re.is_match(f) && !xs.contains(f) {
                                println!("{f}");
                            }
                        }
                    } else {
                        for f in atoms.iter() {
                            if !xs.contains(f) {
                                println!("{f}")
                            }
                        }
                    }
                }
            }

            Some(&_) | None => {
                if let Some(bcs) = nav.nav.brave_consequences(route.iter()) {
                    if bcs.is_empty() {
                        println!("no answer set");
                    } else {
                        let bcs_str = bcs.iter().map(|f| lex::repr(*f)).collect::<Vec<_>>();

                        if let Some(re) = fst.and_then(|s| Regex::new(r#s).ok()) {
                            for f in atoms.iter() {
                                if !re.is_match(f) {
                                    continue;
                                }

                                if !bcs_str.contains(f) {
                                    println!("\x1b[0;30;41m{}\x1b[0m", f);
                                } else {
                                    if let Ok(1) = nav.nav.enumerate_solutions_quietly(
                                        Some(1),
                                        route.iter().chain([format!("~{f}")].iter()),
                                    ) {
                                    } else {
                                        println!("\x1b[0;30;42m{}\x1b[0m", f);
                                    }
                                }
                            }
                        } else {
                            for f in atoms.iter() {
                                if !bcs_str.contains(f) {
                                    println!("\x1b[0;30;41m{}\x1b[0m", f)
                                } else {
                                    if let Ok(1) = nav.nav.enumerate_solutions_quietly(
                                        Some(1),
                                        route.iter().chain([format!("~{f}")].iter()),
                                    ) {
                                    } else {
                                        println!("\x1b[0;30;42m{}\x1b[0m", f)
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        println!("ent time elapsed: {:?}", start.elapsed());
        Ok(())
    }

    #[pyfunction]
    pub fn compute_facets_su(
        nav: &mut PyNavigator,
        atoms: Vec<String>,
        route: Vec<String>,
        args: Vec<String>,
    ) -> PyResult<Vec<String>> {
        let xs = if let Some(re) = args.first().and_then(|s| Regex::new(s).ok()) {
            atoms
                .iter()
                .filter(|a| re.is_match(a))
                .cloned()
                .collect::<Vec<_>>()
        } else {
            atoms.iter().cloned().collect::<Vec<_>>()
        };

        let mut or = ":-".to_owned();
        xs.iter().for_each(|a| {
            or = format!("{or} not {a},");
        });
        or = format!("{}.", &or[..or.len() - 1]);

        let shows = nav
            .nav
            .symbols()
            .filter(|(s, _)| xs.iter().any(|a| a.starts_with(s)))
            .map(|(s, n)| format!("#show {s}/{n}."))
            .collect::<Vec<_>>()
            .join("\n");

        let s = format!("{shows}\n{or}");

        nav.nav
            .add_rule(s.clone())
            .map_err(|_| PyRuntimeError::new_err("compute_facets_su failed"))?;
        nav.invalidate_cache_internal();

        let facets = cached_facets_projecting(nav, &route)?;

        nav.nav
            .remove_rule(s)
            .map_err(|_| PyRuntimeError::new_err("compute_facets_su failed"))?;
        nav.invalidate_cache_internal();

        Ok(facets)
    }

    #[pyfunction]
    pub fn compute_facets_soe_projecting(
        nav: &mut PyNavigator,
        atoms: Vec<String>,
        route: Vec<String>,
        args: Vec<String>,
    ) -> PyResult<Vec<String>> {
        let xs = if let Some(re) = args.first().and_then(|s| Regex::new(s).ok()) {
            atoms
                .iter()
                .filter(|a| re.is_match(a))
                .cloned()
                .collect::<Vec<_>>()
        } else {
            atoms.iter().cloned().collect::<Vec<_>>()
        };
        let shows = nav
            .nav
            .symbols()
            .filter(|(s, _)| xs.iter().any(|a| a.starts_with(s)))
            .map(|(s, n)| format!("#show {s}/{n}."))
            .collect::<Vec<_>>()
            .join("\n");
        nav.nav.add_rule(shows.clone()).unwrap();
        nav.invalidate_cache_internal();
        let cc = nav.nav.cautious_consequences_projecting(route.iter());
        nav.nav.remove_rule(shows).unwrap();
        nav.invalidate_cache_internal();

        let ys = cc
            .map(|cc| {
                let cc_ = cc.iter().map(|s| s.to_string()).collect::<Vec<_>>();
                xs.iter()
                    .filter(move |x| !cc_.contains(x))
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .unwrap();
        let shows = nav
            .nav
            .symbols()
            .filter(|(s, _)| ys.iter().any(|a| a.starts_with(s)))
            .map(|(s, n)| format!("#show {s}/{n}."))
            .collect::<Vec<_>>()
            .join("\n");
        nav.nav.add_rule(shows.clone()).unwrap();
        nav.invalidate_cache_internal();
        nav.nav
            .add_arg("--project=show")
            .map_err(|_| PyRuntimeError::new_err("compute_facets_soe_projecting failed"))?;

        let facets = nav.nav.sieve_quiet(&ys).unwrap();

        nav.nav.remove_rule(shows).unwrap();
        nav.invalidate_cache_internal();
        Ok(facets)
    }

    #[pyfunction]
    pub fn get_is_facet_r(
        nav: &mut PyNavigator,
        atoms: Vec<String>,
        args: Vec<String>,
    ) -> PyResult<Vec<String>> {
        let mut fs = vec![];
        let mut k = 0;
        let xs = if let Some(re) = args.first().and_then(|s| Regex::new(r#s).ok()) {
            atoms.iter().filter(|a| re.is_match(a)).collect::<Vec<_>>()
        } else {
            atoms.iter().collect::<Vec<_>>()
        };
        let (n, mut m) = (atoms.len() as u64, 0);
        let pb = ProgressBar::new(n);
        let style = "{spinner:.green} [{elapsed_precise}] [{wide_bar}] ({eta})";
        pb.set_style(ProgressStyle::with_template(style).unwrap().with_key(
            "eta",
            |state: &ProgressState, w: &mut dyn Write| {
                write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
            },
        ));

        let lp = nav.nav.program();
        let clp = is_facet::copy_program(lp.clone());
        nav.nav
            .add_rule(clp.clone())
            .map_err(|_| PyRuntimeError::new_err("is_facet_r failed"))?;
        nav.invalidate_cache_internal();

        for x in xs {
            if is_facet::is_facet_r(&mut nav.nav, x.to_string()) {
                fs.push(x.to_owned());
                k += 2;
            }
            m += 1;
            pb.set_position(m);
            thread::sleep(Duration::from_millis(12));
        }
        pb.finish_with_message("computed facets");
        println!("\n{k}");
        let facets = fs;

        nav.nav
            .remove_rule(clp)
            .map_err(|_| PyRuntimeError::new_err("is_facet_r failed"))?;
        nav.invalidate_cache_internal();

        Ok(facets)
    }

    // this is "is_facet" in Mode enum
    #[pyfunction]
    pub fn get_is_facet(nav: &mut PyNavigator, args: Vec<String>) -> PyResult<()> {
        if let Some(x) = args.first().and_then(|s| Regex::new(r#s).ok()) {
            println!("{:?}", is_facet::is_facet(&mut nav.nav, x.to_string()))
        }

        Ok(())
    }

    #[pyfunction]
    pub fn get_weighted_facet_count(
        nav: &mut PyNavigator,
        route: Vec<String>,
        args: Vec<String>,
    ) -> PyResult<()> {
        match args
            .first()
            .and_then(|filename| parse_weighted_facets_from_file(filename))
            .and_then(|wfcs| weighted_facet_count(&mut nav.nav, route.to_vec(), wfcs))
        {
            Some(score) => println!("{:?}", score),
            _ => println!("NA"),
        }
        Ok(())
    }

    #[pyfunction]
    pub fn weighted_facet_counts(
        nav: &mut PyNavigator,
        mut route: Vec<String>,
        facets: Vec<String>,
        args: Vec<String>,
    ) -> PyResult<Vec<String>> {
        match args
            .first()
            .and_then(|filename| parse_weighted_facets_from_file(filename))
        {
            Some(wfcs) => {
                if let Some(re) = args.get(1).and_then(|s| Regex::new(r#s).ok()) {
                    for f in facets.iter().filter(|f| re.is_match(f)) {
                        route.push(f.to_owned());
                        match weighted_facet_count(&mut nav.nav, route.to_vec(), wfcs.clone()) {
                            Some(score) => println!("{:?} {f}", score),
                            _ => println!("NA"),
                        }
                        route.pop();
                        route.push(format!("~{f}"));
                        match weighted_facet_count(&mut nav.nav, route.to_vec(), wfcs.clone()) {
                            Some(score) => println!("{:?} ~{f}", score),
                            _ => println!("NA"),
                        }
                        route.pop();
                    }
                } else {
                    for f in facets.iter() {
                        route.push(f.to_owned());
                        match weighted_facet_count(&mut nav.nav, route.to_vec(), wfcs.clone()) {
                            Some(score) => println!("{:?} {f}", score),
                            _ => println!("NA"),
                        }
                        route.pop();
                        route.push(format!("~{f}"));
                        match weighted_facet_count(&mut nav.nav, route.to_vec(), wfcs.clone()) {
                            Some(score) => println!("{:?} ~{f}", score),
                            _ => println!("NA"),
                        }
                        route.pop();
                    }
                }
            }
            _ => println!("NA"),
        }

        Ok(route)
    }

    #[pyfunction]
    pub fn enumerate_solutions(
        nav: &mut PyNavigator,
        route: Vec<String>,
        args: Vec<String>,
    ) -> PyResult<()> {
        let n = nav
            .nav
            .enumerate_solutions(
                args.first().and_then(|n| n.parse::<usize>().ok()),
                route.iter().chain(args.iter().skip(1)).map(String::as_str),
            )
            .map_err(|_| PyRuntimeError::new_err("enumerate_solutions failed"))?;
        println!("found {:?}", n);

        Ok(())
    }

    #[pyfunction]
    pub fn show_facets(facets: Vec<String>, args: Vec<String>) -> PyResult<()> {
        if let Some(re) = args.first().and_then(|s| Regex::new(r#s).ok()) {
            facets
                .iter()
                .filter(|f| re.is_match(f))
                .for_each(|f| print!("{} ", f));
        } else {
            facets.iter().for_each(|f| print!("{} ", f));
        }
        println!();

        Ok(())
    }

    #[pyfunction]
    pub fn facet_count(facets: Vec<String>) -> PyResult<()> {
        println!("{:?}", 2 * facets.len());

        Ok(())
    }

    #[pyfunction]
    pub fn facet_counts(
        nav: &mut PyNavigator,
        facets: Vec<String>,
        mut route: Vec<String>,
        args: Vec<String>,
    ) -> PyResult<Vec<String>> {
        let ovr_count = (2 * facets.len()) as f32;
        let mut weight = Weight::FacetCounting;

        if let Some(re) = args.first().and_then(|s| Regex::new(r#s).ok()) {
            for f in facets.iter().filter(|f| re.is_match(f)) {
                route.push(f.to_owned());
                count(&mut weight, &mut nav.nav, route.iter())
                    .map(|c| println!("{:.4} {:?} {f}", c, 1.0 - (c as f32 / ovr_count)))
                    .ok_or(PyRuntimeError::new_err("facet_counts failed"))?;
                route.pop();
                route.push(format!("~{f}"));
                count(&mut weight, &mut nav.nav, route.iter())
                    .map(|c| println!("{:.4} {:?} ~{f}", c, 1.0 - (c as f32 / ovr_count)))
                    .ok_or(PyRuntimeError::new_err("facet_counts failed"))?;
                route.pop();
            }
        } else {
            for f in facets.iter() {
                route.push(f.to_owned());
                count(&mut weight, &mut nav.nav, route.iter())
                    .map(|c| println!("{:.4} {:?} {f}", 1.0 - (c as f32 / ovr_count), c))
                    .ok_or(PyRuntimeError::new_err("facet_counts failed"))?;
                route.pop();
                route.push(format!("~{f}"));
                count(&mut weight, &mut nav.nav, route.iter())
                    .map(|c| println!("{:.4} {:?} ~{f}", 1.0 - (c as f32 / ovr_count), c))
                    .ok_or(PyRuntimeError::new_err("facet_counts failed"))?;
                route.pop();
            }
        }

        Ok(route)
    }

    #[pyfunction]
    pub fn facet_counts_projecting(
        nav: &mut PyNavigator,
        atoms: Vec<String>,
        facets: Vec<String>,
        mut route: Vec<String>,
        args: Vec<String>,
    ) -> PyResult<Vec<String>> {
        let ovr_count = (2 * facets.len()) as f32;
        let mut weight = Weight::FacetCounting;

        let xs = if let Some(re) = args.first().and_then(|s| Regex::new(r#s).ok()) {
            atoms
                .iter()
                .filter(|a| re.is_match(a))
                .cloned()
                .collect::<Vec<_>>()
        } else {
            atoms.iter().cloned().collect::<Vec<_>>()
        };

        let mut or = ":-".to_owned();
        xs.iter().for_each(|a| {
            or = format!("{or} not {a},");
        });
        or = format!("{}.", &or[..or.len() - 1]);

        let shows = nav
            .nav
            .symbols()
            .filter(|(s, _)| xs.iter().any(|a| a.starts_with(s)))
            .map(|(s, n)| format!("#show {s}/{n}."))
            .collect::<Vec<_>>()
            .join("\n");

        let s = format!("{shows}\n{or}");

        nav.nav
            .add_rule(s.clone())
            .map_err(|_| PyRuntimeError::new_err("enumerate_solutions failed"))?;
        nav.invalidate_cache_internal();

        if let Some(re) = args.get(1).and_then(|s| Regex::new(r#s).ok()) {
            for f in facets.iter().filter(|f| re.is_match(f)) {
                route.push(f.to_owned());
                count_projecting(&mut weight, &mut nav.nav, route.iter())
                    .map(|c| println!("{:.4} {:?} {f}", c, 1.0 - (c as f32 / ovr_count)))
                    .ok_or(PyRuntimeError::new_err("facet_counts_projecting failed"))?;
                route.pop();
                route.push(format!("~{f}"));
                count_projecting(&mut weight, &mut nav.nav, route.iter())
                    .map(|c| println!("{:.4} {:?} ~{f}", c, 1.0 - (c as f32 / ovr_count)))
                    .ok_or(PyRuntimeError::new_err("facet_counts_projecting failed"))?;
                route.pop();
            }
        } else {
            for f in facets.iter() {
                route.push(f.to_owned());
                count_projecting(&mut weight, &mut nav.nav, route.iter())
                    .map(|c| println!("{:.4} {:?} {f}", 1.0 - (c as f32 / ovr_count), c))
                    .ok_or(PyRuntimeError::new_err("facet_counts_projecting failed"))?;
                route.pop();
                route.push(format!("~{f}"));
                count_projecting(&mut weight, &mut nav.nav, route.iter())
                    .map(|c| println!("{:.4} {:?} ~{f}", 1.0 - (c as f32 / ovr_count), c))
                    .ok_or(PyRuntimeError::new_err("facet_counts_projecting failed"))?;
                route.pop();
            }
        }

        nav.nav
            .remove_rule(s)
            .map_err(|_| PyRuntimeError::new_err("enumerate_solutions failed"))?;
        nav.invalidate_cache_internal();

        Ok(route)
    }

    #[pyfunction]
    pub fn answer_set_count(
        nav: &mut PyNavigator,
        route: Vec<String>,
        args: Vec<String>,
    ) -> PyResult<()> {
        let n = nav
            .nav
            .enumerate_solutions_quietly(
                args.first().and_then(|n| n.parse::<usize>().ok()),
                route.iter().chain(args.iter().skip(1)).map(String::as_str),
            )
            .map_err(|_| PyRuntimeError::new_err("answer_set_count failed"))?;

        println!("{:?}", n);

        Ok(())
    }

    #[pyfunction]
    pub fn answer_set_counts(
        nav: &mut PyNavigator,
        facets: Vec<String>,
        mut route: Vec<String>,
        args: Vec<String>,
    ) -> PyResult<Vec<String>> {
        let mut weight = Weight::AnswerSetCounting;
        let ovr_count = (count(&mut weight, &mut nav.nav, route.iter())
            .ok_or(PyRuntimeError::new_err("answer_set_counts failed"))?)
            as f32;

        if let Some(re) = args.first().and_then(|s| Regex::new(r#s).ok()) {
            for f in facets.iter().filter(|f| re.is_match(f)) {
                route.push(f.to_owned());
                count(&mut weight, &mut nav.nav, route.iter())
                    .map(|c| println!("{:.4} {:?} {f}", 1.0 - (c as f32 / ovr_count), c))
                    .ok_or(PyRuntimeError::new_err("answer_set_counts failed"))?;
                route.pop();
                route.push(format!("~{f}"));
                count(&mut weight, &mut nav.nav, route.iter())
                    .map(|c| println!("{:.4} {:?} ~{f}", 1.0 - (c as f32 / ovr_count), c))
                    .ok_or(PyRuntimeError::new_err("answer_set_counts failed"))?;
                route.pop();
            }
        } else {
            for f in facets.iter() {
                route.push(f.to_owned());
                count(&mut weight, &mut nav.nav, route.iter())
                    .map(|c| println!("{:.4} {:?} {f}", 1.0 - (c as f32 / ovr_count), c))
                    .ok_or(PyRuntimeError::new_err("answer_set_counts failed"))?;
                route.pop();
                route.push(format!("~{f}"));
                count(&mut weight, &mut nav.nav, route.iter())
                    .map(|c| println!("{:.4} {:?} ~{f}", 1.0 - (c as f32 / ovr_count), c))
                    .ok_or(PyRuntimeError::new_err("answer_set_counts failed"))?;
                route.pop();
            }
        }

        Ok(route)
    }

    #[pyfunction]
    pub fn show_route(route: Vec<String>, ctx: Vec<String>) -> PyResult<()> {
        if let Some(f) = ctx.first() {
            println!("{f}")
        }

        route.iter().for_each(|f| print!("{f} "));
        println!();

        Ok(())
    }

    #[pyfunction]
    pub fn del_last(nav: &mut PyNavigator, mut route: Vec<String>) -> PyResult<Vec<String>> {
        route.pop();

        let facets = cached_facets(nav, &route)?;

        Ok(facets)
    }

    #[pyfunction]
    pub fn clear_route(
        nav: &mut PyNavigator,
        mut route: Vec<String>,
    ) -> PyResult<(Vec<String>, Vec<String>)> {
        route.clear();

        let facets = cached_facets(nav, &route)?;

        Ok((route, facets))
    }

    // skip display_mode()

    // skip change_mode()

    // TODO: check return type again
    #[pyfunction]
    pub fn propose_step(
        nav: &mut PyNavigator,
        facets: Vec<String>,
        route: Vec<String>,
        args: Vec<String>,
    ) -> PyResult<Vec<String>> {
        let fs = if let Some(re) = args.first().and_then(|s| Regex::new(r#s).ok()) {
            facets
                .iter()
                .filter(|f| re.is_match(f))
                .cloned()
                .collect::<Vec<_>>()
        } else {
            facets.to_vec()
        };

        let mode = Mode::GoalOriented(None::<usize>);
        let mut py_mode = ModeOptionUsize { inner: mode };

        let mut return_active: Vec<String> = vec![];

        match propose_next_step_option_usize(&mut py_mode, nav, route, fs) {
            Ok((active, Some((f, Some(c))))) => {
                println!("{f} {:?}", c);
                return_active = active
            }
            Ok((active, Some((f, None)))) => {
                println!("{f} _");
                return_active = active
            }
            _ => println!("noop"),
        }

        Ok(return_active)
    }

    // TODO: check return type again
    #[pyfunction]
    pub fn take_step(
        nav: &mut PyNavigator,
        mut facets: Vec<String>,
        route: Vec<String>,
        args: Vec<String>,
    ) -> PyResult<(Vec<String>, Vec<String>)> {
        let start = Instant::now();
        let fs = if let Some(re) = args.first().and_then(|s| Regex::new(r#s).ok()) {
            facets
                .iter()
                .filter(|f| re.is_match(f))
                .cloned()
                .collect::<Vec<_>>()
        } else {
            facets.to_vec()
        };

        let ovr_count = (usize::default()) as f32;
        let mode = Mode::GoalOriented(None::<usize>);
        let mut py_mode = ModeOptionUsize { inner: mode };

        let mut return_active: Vec<String> = vec![];

        match perform_next_step_option_usize(&mut py_mode, nav, route.clone(), fs) {
            Ok((active, Some((f, Some(c))))) => {
                println!("{:.4} {:?} {f}", 1.0 - (c as f32 / ovr_count), c);
                // self.update(Some(c));
                return_active = active;
                facets = cached_facets(nav, &return_active)?;
            }
            Ok((active, Some((f, None)))) => {
                println!("_ _ {f}");
                return_active = active;
                facets = cached_facets(nav, &return_active)?;
            }
            _ => println!("noop"),
        }

        println!("tak time elapsed: {:?}", start.elapsed());
        Ok((facets, return_active))
    }

    #[pyfunction]
    pub fn execute_loop(
        nav: &mut PyNavigator,
        mut atoms: Vec<String>,
        mut facets: Vec<String>,
        mut route: Vec<String>,
        args: Vec<String>,
        mut ctx: Vec<String>,
    ) -> PyResult<(Vec<String>, Vec<String>, Vec<String>, Vec<String>)> {
        let tmp: Vec<String> = args.into_iter().map(|s| s.replace('\\', "")).collect();
        let joined = tmp.join(" ");

        let mut parts = joined.splitn(2, WHILE_LOOP_DO);
        let condition_part = parts.next().unwrap_or("").trim();
        let body_part = parts.next().unwrap_or("").trim();

        if condition_part.is_empty() || body_part.is_empty() {
            println!(
                "error: expected '{} <condition> {} <commands>'",
                LOOP, WHILE_LOOP_DO
            );
            return Ok((atoms, facets, route, ctx));
        }

        let cond_tokens: Vec<&str> = condition_part.split_whitespace().collect();
        if cond_tokens.len() != 3 {
            println!("error: condition must be of form <var> <op> <number>");
            return Ok((atoms, facets, route, ctx));
        }
        let var = cond_tokens[0];
        let op = cond_tokens[1];
        let rhs = cond_tokens[2].parse::<usize>().ok();
        let rhs = match rhs {
            Some(n) => n,
            None => {
                println!("error: right-hand side must be a positive integer");
                return Ok((atoms, facets, route, ctx));
            }
        };

        let commands: Vec<&str> = body_part.split(WHILE_LOOP_CMD_SEP).collect();

        let condition_holds = |facets_len: usize, route_len: usize| -> bool {
            let lhs = match var {
                WHILE_LOOP_VAR_FACETS => facets_len,
                WHILE_LOOP_VAR_ROUTE => route_len,
                _ => {
                    println!("error: unknown variable '{}'", var);
                    return false;
                }
            };
            match op {
                WHILE_LOOP_OP_NEQ => lhs != rhs,
                WHILE_LOOP_OP_GT => lhs > rhs,
                WHILE_LOOP_OP_GTE => lhs >= rhs,
                WHILE_LOOP_OP_LT => lhs < rhs,
                WHILE_LOOP_OP_LTE => lhs <= rhs,
                _ => {
                    println!("error: unknown operator '{}'", op);
                    false
                }
            }
        };

        while !facets.is_empty() && condition_holds(2 * facets.len(), route.len()) {
            for cmd in &commands {
                let cmd_str = cmd.trim().to_string();
                (atoms, facets, route, ctx) = command(
                    cmd_str,
                    nav,
                    atoms.clone(),
                    facets.clone(),
                    route.clone(),
                    ctx.clone(),
                )?;
            }
        }

        Ok((atoms, facets, route, ctx))
    }
    #[pyfunction]
    pub fn is_atom(nav: &mut PyNavigator, args: Vec<String>) -> PyResult<()> {
        match args.first().and_then(|a| nav.nav.is_known(a.to_owned())) {
            Some(v) => println!("{v}"),
            _ => println!("error: invalid atom"),
        }

        Ok(())
    }

    #[pyfunction]
    pub fn show_atoms(nav: &mut PyNavigator) -> PyResult<()> {
        nav.nav.atoms().for_each(|a| {
            print!("{a} ");
        });

        println!();

        Ok(())
    }

    #[pyfunction]
    pub fn filter_atoms(args: Vec<String>, atoms: Vec<String>) -> PyResult<()> {
        let mut k = 0;
        if let Some(re) = args.first().and_then(|s| Regex::new(r#s).ok()) {
            atoms.iter().filter(|f| re.is_match(f)).for_each(|f| {
                k += 1;
                print!("{} ", f)
            });
        } else {
            atoms.iter().for_each(|f| {
                k += 1;
                print!("{} ", f)
            });
        }

        println!("\n{k}");

        Ok(())
    }

    #[pyfunction]
    pub fn show_program(nav: &mut PyNavigator) -> PyResult<()> {
        println!("{}", nav.nav.program());

        Ok(())
    }

    #[pyfunction]
    pub fn sieve_facets(
        nav: &mut PyNavigator,
        facets: Vec<String>,
        args: Vec<String>,
    ) -> PyResult<()> {
        let fs = if let Some(re) = args.first().and_then(|s| Regex::new(r#s).ok()) {
            facets
                .iter()
                .filter(|f| re.is_match(f))
                .cloned()
                .collect::<Vec<_>>()
        } else {
            facets.to_vec()
        };
        nav.nav
            .sieve(&fs)
            .map_err(|_| PyRuntimeError::new_err("sieve_facets failed"))?;

        Ok(())
    }

    #[pyfunction]
    pub fn context(
        nav: &mut PyNavigator,
        route: Vec<String>,
        args: Vec<String>,
        mut ctx: Vec<String>,
    ) -> PyResult<(Vec<String>, Vec<String>)> {
        if ctx.len() > 1 {
            for r in ctx.drain(1..) {
                unsafe { nav.nav.remove_rule(r).unwrap_unchecked() };
                nav.invalidate_cache_internal();
            }
        }

        ctx.clear();

        if let Some(cnf) = args.first() {
            ctx.push(cnf.to_string());

            let clauses = cnf.split("&");
            for clause in clauses {
                let body = clause
                    .split("|")
                    .map(|lit| match lit.starts_with('~') {
                        true => lit[1..].to_owned(),
                        _ => format!("not {lit}"),
                    })
                    .collect::<Vec<_>>()
                    .join(",");

                let ic = format!(":- {body}. ");

                ctx.push(ic.clone());

                nav.nav
                    .add_rule(ic)
                    .map_err(|_| PyRuntimeError::new_err("context failed"))?;
                nav.invalidate_cache_internal();
            }
        }

        let facets = cached_facets(nav, &route)?;

        Ok((ctx, facets))
    }

    #[pyfunction]
    pub fn significance(
        nav: &mut PyNavigator,
        route: Vec<String>,
        facets: Vec<String>,
        args: Vec<String>,
    ) -> PyResult<()> {
        let start = Instant::now();
        let y = args.first().unwrap();

        if let Some(re) = args.get(1).and_then(|s| Regex::new(r#s).ok()) {
            nav.nav.significance(&route, y.to_owned(), &facets, re)
        }

        println!("sig time elapsed: {:?}", start.elapsed());

        Ok(())
    }

    #[pyfunction]
    pub fn significance_projecting(
        nav: &mut PyNavigator,
        facets: Vec<String>,
        atoms: Vec<String>,
        route: Vec<String>,
        args: Vec<String>,
    ) -> PyResult<()> {
        let y = args.first().unwrap();

        let xs = if let Some(re) = args.get(1).and_then(|s| Regex::new(r#s).ok()) {
            atoms
                .iter()
                .filter(|a| re.is_match(a))
                .cloned()
                .collect::<Vec<_>>()
        } else {
            atoms.iter().cloned().collect::<Vec<_>>()
        };

        let mut or = ":-".to_owned();
        xs.iter().for_each(|a| {
            or = format!("{or} not {a},");
        });
        or = format!("{}.", &or[..or.len() - 1]);

        let shows = nav
            .nav
            .symbols()
            .filter(|(s, _)| xs.iter().any(|a| a.starts_with(s)))
            .map(|(s, n)| format!("#show {s}/{n}."))
            .collect::<Vec<_>>()
            .join("\n");

        let s = format!("{shows}\n{or}");

        nav.nav
            .add_rule(s.clone())
            .map_err(|_| PyRuntimeError::new_err("significance_projecting failed"))?;
        nav.invalidate_cache_internal();

        if let Some(re) = args.get(2).and_then(|s| Regex::new(r#s).ok()) {
            nav.nav
                .significance_projecting(&route, y.to_owned(), &facets, re)
        }

        nav.nav
            .remove_rule(s.clone())
            .map_err(|_| PyRuntimeError::new_err("significance_projecting failed"))?;
        nav.invalidate_cache_internal();

        Ok(())
    }

    #[pyfunction]
    pub fn enumerate_projected_solutions(
        nav: &mut PyNavigator,
        args: Vec<String>,
        facets: Vec<String>,
        route: Vec<String>,
    ) -> PyResult<()> {
        let n = nav
            .nav
            .enumerate_projected_solutions(
                args.first().and_then(|n| n.parse::<usize>().ok()).take(),
                route.iter().chain(args.iter().skip(1)).map(String::as_str),
                facets.clone(),
            )
            .map_err(|_| PyRuntimeError::new_err("enumerate_projected_solutions failed"))?;

        println!("found {:?}", n);

        Ok(())
    }

    #[pyfunction]
    pub fn handle_unknown(cmd: &str) -> PyResult<()> {
        if cmd.starts_with("//") {
            return Ok(());
        }

        println!("noop [unknown command]");

        Ok(())
    }

    #[pyfunction]
    pub fn cache_control(nav: &mut PyNavigator, args: Vec<String>) -> PyResult<()> {
        match args.first().map(String::as_str) {
            Some(CACHE_ON) => {
                nav.set_cache_enabled_internal(true);
                println!("cache enabled");
            }
            Some(CACHE_OFF) => {
                nav.set_cache_enabled_internal(false);
                println!("cache disabled");
            }
            Some(CACHE_CLEAR) => {
                nav.clear_cache_internal();
                println!("cache cleared");
            }
            Some(CACHE_SIZE) => match args.get(1).and_then(|value| value.parse::<usize>().ok()) {
                Some(capacity) => {
                    let capacity = NonZeroUsize::new(capacity).ok_or_else(|| {
                        PyRuntimeError::new_err("cache size must be greater than zero")
                    })?;
                    nav.set_cache_capacity_internal(capacity);
                    println!("cache size set to {}", nav.facet_cache.capacity());
                }
                None => {
                    println!("cache size: {}", nav.facet_cache.capacity());
                }
            },
            Some(CACHE_STATUS) | None => {
                let status = if nav.cache_enabled {
                    "enabled"
                } else {
                    "disabled"
                };
                println!(
                    "cache {status}; entries: {}; capacity: {}; program_version: {}",
                    nav.facet_cache.len(),
                    nav.facet_cache.capacity(),
                    nav.program_version
                );
            }
            Some(_) => {
                println!(
                    "usage: {CACHE} {{{CACHE_ON},{CACHE_OFF},{CACHE_CLEAR},{CACHE_SIZE},{CACHE_STATUS}}}"
                );
            }
        }

        Ok(())
    }

    #[pyfunction]
    pub fn command(
        expr: String,
        nav: &mut PyNavigator,
        mut atoms: Vec<String>,
        mut facets: Vec<String>,
        mut route: Vec<String>,
        mut ctx: Vec<String>,
    ) -> PyResult<(Vec<String>, Vec<String>, Vec<String>, Vec<String>)> {
        let mut parts = expr.as_str().split_whitespace();
        let command = parts.next();
        let args: Vec<String> = parts.map(String::from).collect();

        match command {
            Some(ACTIVATE_FACETS) | Some(ACTIVATE_FACETS_ALIAS) => {
                (facets, route) = activate_facets(nav, route, args)?;
            }
            Some(ACTIVATE_FACETS_LT) | Some(ACTIVATE_FACETS_LT_ALIAS) => {
                (facets, route) = activate_facets_lt(nav, facets, route, args)?;
            }
            Some(ACTIVATE_FACETS_LAZY) | Some(ACTIVATE_FACETS_LAZY_ALIAS) => {
                route = activate_facets_lazy(route, args)?;
            }
            Some(COMPUTE_FACETS) | Some(COMPUTE_FACETS_ALIAS) => {
                facets = compute_facets(nav, route.clone(), args)?;
            }
            Some(ENTAILMENT) => {
                entailment(nav, atoms.clone(), route.clone(), args)?;
            }
            Some(COMPUTE_FACETS_SU) | Some(COMPUTE_FACETS_SU_ALIAS) => {
                facets = compute_facets_su(nav, atoms.clone(), route.clone(), args)?;
            }
            Some(COMPUTE_FACETS_SOE) => {
                facets = compute_facets_soe_projecting(nav, atoms.clone(), route.clone(), args)?;
            }
            Some(IS_FACET_R) | Some(IS_FACET_R_ALIAS) => {
                facets = get_is_facet_r(nav, atoms.clone(), args)?;
            }
            Some(IS_FACET) | Some(IS_FACET_ALIAS) => {
                get_is_facet(nav, args)?;
            }
            Some(WEIGHTED_FACET_COUNT) | Some(WEIGHTED_FACET_COUNT_ALIAS) => {
                get_weighted_facet_count(nav, route.clone(), args)?;
            }
            Some(WEIGHTED_FACET_COUNTS) | Some(WEIGHTED_FACET_COUNTS_ALIAS) => {
                route = weighted_facet_counts(nav, facets.clone(), route, args)?;
            }
            Some(ENUMERATE_SOLUTIONS) | Some(ENUMERATE_SOLUTIONS_ALIAS) => {
                enumerate_solutions(nav, route.clone(), args)?;
            }
            Some(SHOW_FACETS) | Some(SHOW_FACETS_ALIAS) => {
                show_facets(facets.clone(), args)?;
            }
            Some(FACET_COUNT) | Some(FACET_COUNT_ALIAS) => {
                facet_count(facets.clone())?;
            }
            Some(FACET_COUNTS) | Some(FACET_COUNTS_ALIAS) => {
                route = facet_counts(nav, facets.clone(), route, args)?;
            }
            Some(FACET_COUNTS_PROJECTING) | Some(FACET_COUNTS_PROJECTING_ALIAS) => {
                route = facet_counts_projecting(nav, atoms.clone(), facets.clone(), route, args)?;
            }
            Some(ANSWER_SET_COUNT) | Some(ANSWER_SET_COUNT_ALIAS) => {
                answer_set_count(nav, route.clone(), args)?;
            }
            Some(ANSWER_SET_COUNTS) | Some(ANSWER_SET_COUNTS_ALIAS) => {
                route = answer_set_counts(nav, facets.clone(), route, args)?;
            }
            Some(SHOW_ROUTE) | Some(SHOW_ROUTE_ALIAS) => {
                show_route(route.clone(), ctx.clone())?;
            }
            Some(DEL_LAST) | Some(DEL_LAST_ALIAS) => {
                facets = del_last(nav, route.clone())?;
            }
            Some(CLEAR_ROUTE) | Some(CLEAR_ROUTE_ALIAS) => {
                (route, facets) = clear_route(nav, route)?;
            }
            Some(PROPOSE_STEP) | Some(PROPOSE_STEP_ALIAS) => {
                route = propose_step(nav, facets.clone(), route, args)?;
            }
            Some(TAKE_STEP) | Some(TAKE_STEP_ALIAS) => {
                (facets, route) = take_step(nav, facets, route, args)?;
            }
            Some(QUIT) => {
                std::process::exit(0);
            }
            Some(MANUAL) => {
                crate::config::manual();
            }
            Some(LOOP) => {
                (atoms, facets, route, ctx) = execute_loop(
                    nav,
                    atoms.clone(),
                    facets.clone(),
                    route.clone(),
                    args,
                    ctx.clone(),
                )?;
            }
            Some(IS_ATOM) => {
                is_atom(nav, args)?;
            }
            Some(SHOW_ATOMS) => {
                show_atoms(nav)?;
            }
            Some(FILTER_ATOMS) => {
                filter_atoms(args, atoms.clone())?;
            }
            Some(SHOW_PROGRAM) => {
                show_program(nav)?;
            }
            Some(SOE) => {
                sieve_facets(nav, facets.clone(), args)?;
            }
            Some(CONTEXT) | Some(CONTEXT_ALIAS) => {
                (ctx, facets) = context(nav, route.clone(), args, ctx)?;
            }
            Some(CACHE) | Some(CACHE_ALIAS) => {
                cache_control(nav, args)?;
            }
            Some(SIGNIFICANCE) | Some(SIGNIFICANCE_ALIAS) => {
                significance(nav, route.clone(), facets.clone(), args)?;
            }
            Some(SIGNIFICANCE_PROJECTING) | Some(SIGNIFICANCE_PROJECTING_ALIAS) => {
                significance_projecting(nav, facets.clone(), atoms.clone(), route.clone(), args)?;
            }
            Some(ENUMERATE_PROJECTED_SOLUTIONS) | Some(ENUMERATE_PROJECTED_SOLUTIONS_ALIAS) => {
                enumerate_projected_solutions(nav, args, facets.clone(), route.clone())?;
            }
            Some(CHANGE_MODE) | Some(CHANGE_MODE_ALIAS) => {
                println!("change mode not fully implemented yet");
            }
            Some(DISPLAY_MODE) => {
                println!("display mode not fully implemented yet");
            }
            None => {
                println!("noop [empty command]");
            }
            Some(cmd) => handle_unknown(cmd)?,
        }
        Ok((atoms, facets, route, ctx))
    }
}
