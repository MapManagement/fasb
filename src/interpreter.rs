use crate::config::*;
use crate::is_facet;
use crate::modes::{perform_next_step, propose_next_step, Mode};
use crate::significance::Significance;
use crate::wfc::parse_weighted_facets_from_file;
use crate::wfc::weighted_facet_count;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use regex::Regex;
use savan::lex;
use savan::nav::{
    errors::{NavigatorError, Result},
    facets::Facets,
    soe::Collect,
    weights::{count, count_projecting, Weight},
    Navigator,
};
use std::fmt::Write;
use std::thread;
use std::time::Duration;
use std::time::Instant;

impl Mode<Option<usize>> {
    pub fn command(
        &mut self,
        expr: String,
        nav: &mut Navigator,
        atoms: &mut Vec<String>,
        facets: &mut Vec<String>,
        route: &mut Vec<String>,
        ctx: &mut Vec<String>,
    ) -> Result<()> {
        let mut parts = expr.split_whitespace();
        let command = parts.next();
        let args: Vec<String> = parts.map(String::from).collect();

        match command {
            Some(ACTIVATE_FACETS) => self.activate_facets(nav, facets, route, args)?,
            Some(ACTIVATE_FACETS_LT) => self.activate_facets_lt(nav, facets, route, args)?,
            Some(ACTIVATE_FACETS_LAZY) => self.activate_facets_lazy(route, args)?,
            Some(COMPUTE_FACETS) => self.compute_facets(nav, facets, route, args)?,
            Some(ENTAILMENT) => self.entailment(nav, atoms, route, args)?,
            Some(COMPUTE_FACETS_SU) => self.compute_facets_su(nav, atoms, facets, route, args)?,
            Some("!?soe") => self.compute_facets_soe_projecting(nav, atoms, facets, route, args)?,
            Some(IS_FACET_R) => self.is_facet_r(nav, atoms, facets, args)?,
            Some(IS_FACET) => self.is_facet(nav, args)?,
            Some(WEIGHTED_FACET_COUNT) => self.weighted_facet_count(nav, route, args)?,
            Some(WEIGHTED_FACET_COUNTS) => self.weighted_facet_counts(nav, facets, route, args)?,
            Some(ENUMERATE_SOLUTIONS) => self.enumerate_solutions(nav, route, args)?,
            Some(SHOW_FACETS) => self.show_facets(facets, args)?,
            Some(FACET_COUNT) => self.facet_count(facets)?,
            Some(FACET_COUNTS) => self.facet_counts(nav, facets, route, args)?,
            Some(FACET_COUNTS_PROJECTING) => self.facet_counts_projecting(nav, atoms, facets, route, args)?,
            Some(ANSWER_SET_COUNT) => self.answer_set_count(nav, route, args)?,
            Some(ANSWER_SET_COUNTS) => self.answer_set_counts(nav, facets, route, args)?,
            Some(SHOW_ROUTE) => self.show_route(route, ctx)?,
            Some(DEL_LAST) => self.del_last(nav, facets, route)?,
            Some(CLEAR_ROUTE) => self.clear_route(nav, facets, route)?,
            Some(DISPLAY_MODE) => self.display_mode()?,
            Some(CHANGE_MODE) => self.change_mode(args)?,
            Some(PROPOSE_STEP) => self.propose_step(nav, facets, route, args)?,
            Some(TAKE_STEP) => self.take_step(nav, facets, route, args)?,
            Some(QUIT) => std::process::exit(0),
            Some("man") => crate::config::manual(),
            Some("\\") => self.execute_loop(nav, atoms, facets, route, args, ctx)?, 
            Some(IS_ATOM) => self.is_atom(nav, args)?,
            Some(SHOW_ATOMS) => self.show_atoms(nav)?,
            Some(FILTER_ATOMS) => self.filter_atoms(args, atoms)?,
            Some(SHOW_PROGRAM) => self.show_program(nav)?,
            Some(SOE) => self.sieve_facets(nav, facets, args)?,
            Some(CONTEXT) => self.context(nav, facets, route, args, ctx)?,
            Some(SIGNIFICANCE) => self.significance(nav, facets, route, args)?,
            Some(SIGNIFICANCE_PROJECTING) => {
                self.significance_projecting(nav, facets, atoms, route, args)?
            }
            Some(ENUMERATE_PROJECTED_SOLUTIONS) => {
                self.enumerate_projected_solutions(nav, args, route, facets)?
            }
            Some(cmd) => self.handle_unknown(cmd)?,
            _ => eprintln!("unknown error"),
        }
        Ok(())
    }

    pub fn activate_facets(
        &mut self,
        nav: &mut Navigator,
        facets: &mut Vec<String>,
        route: &mut Vec<String>,
        args: Vec<String>,
    ) -> Result<()> {
        route.extend(args);

        *facets = nav
            .facet_inducing_atoms(route.iter())
            .ok_or(NavigatorError::None)?
            .iter()
            .map(|f| lex::repr(*f))
            .collect();

        Ok(())
    }

    pub fn activate_facets_lt(
        &mut self,
        nav: &mut Navigator,
        facets: &mut Vec<String>,
        route: &mut Vec<String>,
        args: Vec<String>,
    ) -> Result<()> {
        route.extend(args);

        *facets = nav
            .learned_that(facets, route, None)
            .ok_or(NavigatorError::None)?;

        Ok(())
    }

    pub fn activate_facets_lazy(
        &mut self,
        route: &mut Vec<String>,
        args: Vec<String>,
    ) -> Result<()> {
        route.extend(args);
        Ok(())
    }

    pub fn compute_facets(
        &mut self,
        nav: &mut Navigator,
        facets: &mut Vec<String>,
        route: &mut Vec<String>,
        args: Vec<String>,
    ) -> Result<()> {
        let start = Instant::now();
        *facets = if let Some(re) = args.get(0).and_then(|s| Regex::new(r#s).ok()) {
            nav.facet_inducing_atoms(route.iter())
                .ok_or(NavigatorError::None)?
                .iter()
                .map(|f| lex::repr(*f))
                .filter(|a| re.is_match(&a))
                .collect::<Vec<_>>()
        } else {
            nav.facet_inducing_atoms(route.iter())
                .ok_or(NavigatorError::None)?
                .iter()
                .map(|f| lex::repr(*f))
                .collect()
        };
        println!("time elapsed: {:?}", start.elapsed());
        Ok(())
    }

    pub fn entailment(
        &mut self,
        nav: &mut Navigator,
        atoms: &mut Vec<String>,
        route: &mut Vec<String>,
        args: Vec<String>,
    ) -> Result<()> {
        let start = Instant::now();

        let fst = args.get(0).map(|s| s.as_str());
        let regex = args.get(1).and_then(|s| Regex::new(s).ok());

        match fst {
            Some("%") => {
                if let Some(xs) = nav
                    .cautious_consequences(route.iter())
                    .map(|fs| fs.iter().map(|f| lex::repr(*f)).collect::<Vec<_>>())
                {
                    if let Some(re) = regex {
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
                    .brave_consequences(route.iter())
                    .map(|fs| fs.iter().map(|f| lex::repr(*f)).collect::<Vec<_>>())
                {
                    for f in atoms.iter() {
                        if regex.as_ref().map_or(true, |re| re.is_match(f)) && !xs.contains(f) {
                            println!("{f}");
                        }
                    }
                }
            }

            _ => {
                if let Some(bcs) = nav.brave_consequences(route.iter()) {
                    if bcs.is_empty() {
                        println!("no answer set");
                    } else {
                        let bcs_str = bcs.iter().map(|f| lex::repr(*f)).collect::<Vec<_>>();

                        for f in atoms.iter() {
                            if !regex.as_ref().map_or(true, |re| re.is_match(f)) {
                                continue;
                            }

                            if !bcs_str.contains(f) {
                                println!("\x1b[0;30;41m{}\x1b[0m", f);
                            } else {
                                if let Ok(1) = nav.enumerate_solutions_quietly(
                                    Some(1),
                                    route.iter().chain([format!("~{f}")].iter()),
                                ) {
                                } else {
                                    println!("\x1b[0;30;42m{}\x1b[0m", f);
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

    pub fn compute_facets_su(
        &mut self,
        nav: &mut Navigator,
        atoms: &mut Vec<String>,
        facets: &mut Vec<String>,
        route: &mut Vec<String>,
        args: Vec<String>,
    ) -> Result<()> {
        let xs = if let Some(re) = args.get(0).and_then(|s| Regex::new(s).ok()) {
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
            .symbols()
            .filter(|(s, _)| xs.iter().any(|a| a.starts_with(s)))
            .map(|(s, n)| format!("#show {s}/{n}."))
            .collect::<Vec<_>>()
            .join("\n");

        let s = format!("{shows}\n{or}");

        nav.add_rule(s.clone())?;

        *facets = nav
            .facet_inducing_atoms_projecting(route.iter())
            .ok_or(NavigatorError::None)?
            .iter()
            .map(|f| lex::repr(*f))
            .collect();

        nav.remove_rule(s)?;
        Ok(())
    }

    pub fn compute_facets_soe_projecting(
        &mut self,
        nav: &mut Navigator,
        atoms: &mut Vec<String>,
        facets: &mut Vec<String>,
        route: &mut Vec<String>,
        args: Vec<String>,
    ) -> Result<()> {
        let xs = if let Some(re) = args.get(0).and_then(|s| Regex::new(s).ok()) {
            atoms
                .iter()
                .filter(|a| re.is_match(a))
                .cloned()
                .collect::<Vec<_>>()
        } else {
            atoms.iter().cloned().collect::<Vec<_>>()
        };
        let shows = nav
            .symbols()
            .filter(|(s, _)| xs.iter().any(|a| a.starts_with(s)))
            .map(|(s, n)| format!("#show {s}/{n}."))
            .collect::<Vec<_>>()
            .join("\n");
        nav.add_rule(shows.clone()).unwrap();
        let cc = nav.cautious_consequences_projecting(route.iter());
        nav.remove_rule(shows).unwrap();

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
            .symbols()
            .filter(|(s, _)| ys.iter().any(|a| a.starts_with(s)))
            .map(|(s, n)| format!("#show {s}/{n}."))
            .collect::<Vec<_>>()
            .join("\n");
        nav.add_rule(shows.clone()).unwrap();
        nav.add_arg("--project=show")?;

        *facets = nav.sieve_quiet(&ys).unwrap();

        nav.remove_rule(shows).unwrap();
        Ok(())
    }

    pub fn is_facet_r(
        &mut self,
        nav: &mut Navigator,
        atoms: &mut Vec<String>,
        facets: &mut Vec<String>,
        args: Vec<String>,
    ) -> Result<()> {
        let mut fs = vec![];
        let mut k = 0;
        let xs = if let Some(re) = args.get(0).and_then(|s| Regex::new(r#s).ok()) {
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

        let lp = nav.program();
        let clp = is_facet::copy_program(lp.clone());
        nav.add_rule(clp.clone())?;

        for x in xs {
            if is_facet::is_facet_r(nav, x.to_string()) {
                fs.push(x.to_owned());
                k += 2;
            }
            m += 1;
            pb.set_position(m);
            thread::sleep(Duration::from_millis(12));
        }
        pb.finish_with_message("computed facets");
        println!("\n{k}");
        *facets = fs;

        nav.remove_rule(clp)?;
        Ok(())
    }

    pub fn is_facet(&mut self, nav: &mut Navigator, args: Vec<String>) -> Result<()> {
        if let Some(x) = args.get(0).and_then(|s| Regex::new(r#s).ok()) {
            println!("{:?}", is_facet::is_facet(nav, x.to_string()))
        }
        Ok(())
    }

    pub fn weighted_facet_count(
        &mut self,
        nav: &mut Navigator,
        route: &mut Vec<String>,
        args: Vec<String>,
    ) -> Result<()> {
        match args
            .get(0)
            .and_then(|filename| parse_weighted_facets_from_file(filename))
            .and_then(|wfcs| weighted_facet_count(nav, route.to_vec(), wfcs))
        {
            Some(score) => println!("{:?}", score),
            _ => println!("NA"),
        }
        Ok(())
    }
    pub fn weighted_facet_counts(
        &mut self,
        nav: &mut Navigator,
        route: &mut Vec<String>,
        facets: &mut Vec<String>,
        args: Vec<String>,
    ) -> Result<()> {
        match args
            .get(0)
            .and_then(|filename| parse_weighted_facets_from_file(filename))
        {
            Some(wfcs) => {
                if let Some(re) = args.get(1).and_then(|s| Regex::new(r#s).ok()) {
                    for f in facets.iter().filter(|f| re.is_match(f)) {
                        route.push(f.to_owned());
                        match weighted_facet_count(nav, route.to_vec(), wfcs.clone()) {
                            Some(score) => println!("{:?} {f}", score),
                            _ => println!("NA"),
                        }
                        route.pop();
                        route.push(format!("~{f}"));
                        match weighted_facet_count(nav, route.to_vec(), wfcs.clone()) {
                            Some(score) => println!("{:?} ~{f}", score),
                            _ => println!("NA"),
                        }
                        route.pop();
                    }
                } else {
                    for f in facets.iter() {
                        route.push(f.to_owned());
                        match weighted_facet_count(nav, route.to_vec(), wfcs.clone()) {
                            Some(score) => println!("{:?} {f}", score),
                            _ => println!("NA"),
                        }
                        route.pop();
                        route.push(format!("~{f}"));
                        match weighted_facet_count(nav, route.to_vec(), wfcs.clone()) {
                            Some(score) => println!("{:?} ~{f}", score),
                            _ => println!("NA"),
                        }
                        route.pop();
                    }
                }
            }
            _ => println!("NA"),
        }
        Ok(())
    }
    pub fn enumerate_solutions(
        &mut self,
        nav: &mut Navigator,
        route: &mut Vec<String>,
        args: Vec<String>,
    ) -> Result<()> {
        let n = nav.enumerate_solutions(
            args.first().and_then(|n| n.parse::<usize>().ok()),
            route.iter().chain(args.iter().skip(1)).map(String::as_str),
        )?;
        println!("found {:?}", n);
        Ok(())
    }

    pub fn show_facets(&mut self, facets: &mut Vec<String>, args: Vec<String>) -> Result<()> {
        if let Some(re) = args.get(0).and_then(|s| Regex::new(r#s).ok()) {
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
    pub fn facet_count(&mut self, facets: &mut Vec<String>) -> Result<()> {
        println!("{:?}", 2 * facets.len());
        Ok(())
    }
    pub fn facet_counts(
        &mut self,
        nav: &mut Navigator,
        facets: &mut Vec<String>,
        route: &mut Vec<String>,
        args: Vec<String>,
    ) -> Result<()> {
        let ovr_count = match self {
            Self::MaxWeightedFacetCounting(Some(c)) => *c,
            Self::MinWeightedFacetCounting(Some(c)) => *c,
            _ => 2 * facets.len(),
        } as f32;
        let mut weight = Weight::FacetCounting;

        if let Some(re) = args.get(0).and_then(|s| Regex::new(r#s).ok()) {
            for f in facets.iter().filter(|f| re.is_match(f)) {
                route.push(f.to_owned());
                count(&mut weight, nav, route.iter())
                    .map(|c| println!("{:.4} {:?} {f}", c, 1.0 - (c as f32 / ovr_count)))
                    .ok_or(NavigatorError::None)?;
                route.pop();
                route.push(format!("~{f}"));
                count(&mut weight, nav, route.iter())
                    .map(|c| println!("{:.4} {:?} ~{f}", c, 1.0 - (c as f32 / ovr_count)))
                    .ok_or(NavigatorError::None)?;
                route.pop();
            }
        } else {
            for f in facets.iter() {
                route.push(f.to_owned());
                count(&mut weight, nav, route.iter())
                    .map(|c| println!("{:.4} {:?} {f}", 1.0 - (c as f32 / ovr_count), c))
                    .ok_or(NavigatorError::None)?;
                route.pop();
                route.push(format!("~{f}"));
                count(&mut weight, nav, route.iter())
                    .map(|c| println!("{:.4} {:?} ~{f}", 1.0 - (c as f32 / ovr_count), c))
                    .ok_or(NavigatorError::None)?;
                route.pop();
            }
        }
        Ok(())
    }
    pub fn facet_counts_projecting(
        &mut self,
        nav: &mut Navigator,
        atoms: &mut Vec<String>,
        facets: &mut Vec<String>,
        route: &mut Vec<String>,
        args: Vec<String>,
    ) -> Result<()> {
        let ovr_count = match self {
            Self::MaxWeightedFacetCounting(Some(c)) => *c,
            Self::MinWeightedFacetCounting(Some(c)) => *c,
            _ => 2 * facets.len(),
        } as f32;
        let mut weight = Weight::FacetCounting;

        let xs = if let Some(re) = args.get(0).and_then(|s| Regex::new(r#s).ok()) {
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
            .symbols()
            .filter(|(s, _)| xs.iter().any(|a| a.starts_with(s)))
            .map(|(s, n)| format!("#show {s}/{n}."))
            .collect::<Vec<_>>()
            .join("\n");

        let s = format!("{shows}\n{or}");

        nav.add_rule(s.clone())?;

        if let Some(re) = args.get(1).and_then(|s| Regex::new(r#s).ok()) {
            for f in facets.iter().filter(|f| re.is_match(f)) {
                route.push(f.to_owned());
                count_projecting(&mut weight, nav, route.iter())
                    .map(|c| println!("{:.4} {:?} {f}", c, 1.0 - (c as f32 / ovr_count)))
                    .ok_or(NavigatorError::None)?;
                route.pop();
                route.push(format!("~{f}"));
                count_projecting(&mut weight, nav, route.iter())
                    .map(|c| println!("{:.4} {:?} ~{f}", c, 1.0 - (c as f32 / ovr_count)))
                    .ok_or(NavigatorError::None)?;
                route.pop();
            }
        } else {
            for f in facets.iter() {
                route.push(f.to_owned());
                count_projecting(&mut weight, nav, route.iter())
                    .map(|c| println!("{:.4} {:?} {f}", 1.0 - (c as f32 / ovr_count), c))
                    .ok_or(NavigatorError::None)?;
                route.pop();
                route.push(format!("~{f}"));
                count_projecting(&mut weight, nav, route.iter())
                    .map(|c| println!("{:.4} {:?} ~{f}", 1.0 - (c as f32 / ovr_count), c))
                    .ok_or(NavigatorError::None)?;
                route.pop();
            }
        }

        nav.remove_rule(s)?;
        Ok(())
    }

    pub fn answer_set_count(
        &mut self,
        nav: &mut Navigator,
        route: &mut Vec<String>,
        args: Vec<String>,
    ) -> Result<()> {
        let n = nav.enumerate_solutions_quietly(
            args.first().and_then(|n| n.parse::<usize>().ok()),
            route.iter().chain(args.iter().skip(1)).map(String::as_str),
        )?;
        println!("{:?}", n);
        Ok(())
    }

    pub fn answer_set_counts(
        &mut self,
        nav: &mut Navigator,
        facets: &mut Vec<String>,
        route: &mut Vec<String>,
        args: Vec<String>,
    ) -> Result<()> {
        if let Some(re) = args.get(0).and_then(|s| Regex::new(r#s).ok()) {
            let mut weight = Weight::AnswerSetCounting;
            let ovr_count = match self {
                Self::MaxWeightedAnswerSetCounting(Some(c)) => *c,
                Self::MinWeightedAnswerSetCounting(Some(c)) => *c,
                _ => count(&mut weight, nav, route.iter()).ok_or(NavigatorError::None)?,
            } as f32;
            for f in facets.iter().filter(|f| re.is_match(f)) {
                route.push(f.to_owned());
                count(&mut weight, nav, route.iter())
                    .map(|c| println!("{:.4} {:?} {f}", 1.0 - (c as f32 / ovr_count), c))
                    .ok_or(NavigatorError::None)?;
                route.pop();
                route.push(format!("~{f}"));
                count(&mut weight, nav, route.iter())
                    .map(|c| println!("{:.4} {:?} ~{f}", 1.0 - (c as f32 / ovr_count), c))
                    .ok_or(NavigatorError::None)?;
                route.pop();
            }
        } else {
            let mut weight = Weight::AnswerSetCounting;
            let ovr_count = match self {
                Self::MaxWeightedAnswerSetCounting(Some(c)) => *c,
                Self::MinWeightedAnswerSetCounting(Some(c)) => *c,
                _ => count(&mut weight, nav, route.iter()).ok_or(NavigatorError::None)?,
            } as f32;
            for f in facets.iter() {
                route.push(f.to_owned());
                count(&mut weight, nav, route.iter())
                    .map(|c| println!("{:.4} {:?} {f}", 1.0 - (c as f32 / ovr_count), c))
                    .ok_or(NavigatorError::None)?;
                route.pop();
                route.push(format!("~{f}"));
                count(&mut weight, nav, route.iter())
                    .map(|c| println!("{:.4} {:?} ~{f}", 1.0 - (c as f32 / ovr_count), c))
                    .ok_or(NavigatorError::None)?;
                route.pop();
            }
        }
        Ok(())
    }

    pub fn show_route(&mut self, route: &mut Vec<String>, ctx: &mut Vec<String>) -> Result<()> {
        if !ctx.is_empty() {
            ctx.first().map(|f| println!("{f}"));
        }
        route.iter().for_each(|f| print!("{f} "));
        println!();
        Ok(())
    }

    pub fn del_last(
        &mut self,
        nav: &mut Navigator,
        facets: &mut Vec<String>,
        route: &mut Vec<String>,
    ) -> Result<()> {
        route.pop();
        *facets = nav
            .facet_inducing_atoms(route.iter())
            .ok_or(NavigatorError::None)?
            .iter()
            .map(|f| lex::repr(*f))
            .collect();
        Ok(())
    }
    pub fn clear_route(
        &mut self,
        nav: &mut Navigator,
        facets: &mut Vec<String>,
        route: &mut Vec<String>,
    ) -> Result<()> {
        route.clear();
        *facets = nav
            .facet_inducing_atoms(route.iter())
            .ok_or(NavigatorError::None)?
            .iter()
            .map(|f| lex::repr(*f))
            .collect();
        Ok(())
    }

    pub fn display_mode(&mut self) -> Result<()> {
        println!("{}", self);
        Ok(())
    }

    pub fn change_mode(&mut self, args: Vec<String>) -> Result<()> {
        let n = args.get(1).and_then(|n| n.parse::<usize>().ok());
        match args.first().map(String::as_str) {
            Some("min#f") => *self = Mode::MinWeightedFacetCounting(n),
            Some("max#f") => *self = Mode::MaxWeightedFacetCounting(n),
            Some("min#a") => *self = Mode::MinWeightedAnswerSetCounting(n),
            Some("max#a") => *self = Mode::MaxWeightedAnswerSetCounting(n),
            Some("go") => *self = Mode::GoalOriented(n),
            _ => println!("error: specify mode among {{min,max}}#{{f,a,s}}, go}}"),
        }
        Ok(())
    }

    pub fn propose_step(
        &mut self,
        nav: &mut Navigator,
        facets: &mut Vec<String>,
        route: &mut Vec<String>,
        args: Vec<String>,
    ) -> Result<()> {
        let fs = if let Some(re) = args.get(0).and_then(|s| Regex::new(r#s).ok()) {
            facets
                .iter()
                .filter(|f| re.is_match(f))
                .cloned()
                .collect::<Vec<_>>()
        } else {
            facets.to_vec()
        };
        match propose_next_step(self, nav, route, &fs) {
            Some((f, Some(c))) => println!("{f} {:?}", c),
            Some((f, None)) => println!("{f} _"),
            _ => println!("noop"),
        }
        Ok(())
    }

    pub fn take_step(
        &mut self,
        nav: &mut Navigator,
        facets: &mut Vec<String>,
        route: &mut Vec<String>,
        args: Vec<String>,
    ) -> Result<()> {
        let start = Instant::now();
        let fs = if let Some(re) = args.get(0).and_then(|s| Regex::new(r#s).ok()) {
            facets
                .iter()
                .filter(|f| re.is_match(f))
                .cloned()
                .collect::<Vec<_>>()
        } else {
            facets.to_vec()
        };
        let ovr_count = match self {
            Self::MaxWeightedFacetCounting(Some(c)) => *c,
            Self::MinWeightedFacetCounting(Some(c)) => *c,
            Self::MaxWeightedFacetCounting(None) | Self::MinWeightedFacetCounting(None) => {
                2 * facets.len()
            }
            Self::MaxWeightedAnswerSetCounting(Some(c)) => *c,
            Self::MinWeightedAnswerSetCounting(Some(c)) => *c,
            Self::MaxWeightedAnswerSetCounting(None) | Self::MinWeightedAnswerSetCounting(None) => {
                count(&mut Weight::AnswerSetCounting, nav, route.iter())
                    .ok_or(NavigatorError::None)?
            }
            Self::GoalOriented(_) => usize::default(),
        } as f32;

        match perform_next_step(self, nav, route, &fs) {
            Some((f, Some(c))) => {
                println!("{:.4} {:?} {f}", 1.0 - (c as f32 / ovr_count), c);
                self.update(Some(c));
                *facets = nav
                    .facet_inducing_atoms(route.iter())
                    .ok_or(NavigatorError::None)?
                    .iter()
                    .map(|f| lex::repr(*f))
                    .collect();
            }
            Some((f, None)) => {
                println!("_ _ {f}");
                *facets = nav
                    .facet_inducing_atoms(route.iter())
                    .ok_or(NavigatorError::None)?
                    .iter()
                    .map(|f| lex::repr(*f))
                    .collect();
            }
            _ => println!("noop"),
        }

        println!("tak time elapsed: {:?}", start.elapsed());
        Ok(())
    }

    pub fn execute_loop(
        &mut self,
        nav: &mut Navigator,
        atoms: &mut Vec<String>,
        facets: &mut Vec<String>,
        route: &mut Vec<String>,
        args: Vec<String>,
        ctx: &mut Vec<String>,
    ) -> Result<()> {
        let tmp: Vec<String> = args.into_iter().map(|s| s.replace('\\', "")).collect();
        let joined = tmp.join(" ");
        let mut src = joined.trim().split(" | ");

        let mut pred = match src.next() {
            Some(expr) => expr.split(" "),
            _ => {
                println!("error: specify condition");
                return Ok(());
            }
        };
        let inst = match src.next() {
            Some(expr) => expr.split(".").collect::<Vec<_>>(),
            _ => {
                println!("error: found no instructions");
                return Ok(());
            }
        };

        match pred.next() {
            Some("!=") => match pred.next() {
                Some("#f") => match pred.next().and_then(|n| n.parse::<usize>().ok()) {
                    Some(x) => {
                        while !facets.is_empty() && 2 * facets.len() != x {
                            for cmd in &inst {
                                self.command(cmd.trim().to_owned(), nav, atoms, facets, route, ctx)?
                            }
                        }
                    }
                    _ => {
                        println!("error: unknown rhs");
                        return Ok(());
                    }
                },
                Some("#r") => match pred.next().and_then(|n| n.parse::<usize>().ok()) {
                    Some(x) => {
                        while !facets.is_empty() && route.len() != x {
                            for cmd in &inst {
                                self.command(cmd.trim().to_owned(), nav, atoms, facets, route, ctx)?
                            }
                        }
                    }
                    _ => {
                        println!("error: unknown rhs");
                        return Ok(());
                    }
                },
                _ => {
                    println!("error: unknown lhs");
                    return Ok(());
                }
            },
            Some(">") => match pred.next() {
                Some("#f") => match pred.next().and_then(|n| n.parse::<usize>().ok()) {
                    Some(x) => {
                        while !facets.is_empty() && 2 * facets.len() > x {
                            for cmd in &inst {
                                self.command(cmd.trim().to_owned(), nav, atoms, facets, route, ctx)?
                            }
                        }
                    }
                    _ => {
                        println!("error: unknown rhs");
                        return Ok(());
                    }
                },
                Some("#r") => match pred.next().and_then(|n| n.parse::<usize>().ok()) {
                    Some(x) => {
                        while !facets.is_empty() && route.len() > x {
                            for cmd in &inst {
                                self.command(cmd.trim().to_owned(), nav, atoms, facets, route, ctx)?
                            }
                        }
                    }
                    _ => {
                        println!("error: unknown rhs");
                        return Ok(());
                    }
                },
                _ => {
                    println!("error: unknown lhs");
                    return Ok(());
                }
            },
            Some(">=") => match pred.next() {
                Some("#f") => match pred.next().and_then(|n| n.parse::<usize>().ok()) {
                    Some(x) => {
                        while !facets.is_empty() && 2 * facets.len() >= x {
                            for cmd in &inst {
                                self.command(cmd.trim().to_owned(), nav, atoms, facets, route, ctx)?
                            }
                        }
                    }
                    _ => {
                        println!("error: unknown rhs");
                        return Ok(());
                    }
                },
                Some("#r") => match pred.next().and_then(|n| n.parse::<usize>().ok()) {
                    Some(x) => {
                        while !facets.is_empty() && route.len() >= x {
                            for cmd in &inst {
                                self.command(cmd.trim().to_owned(), nav, atoms, facets, route, ctx)?
                            }
                        }
                    }
                    _ => {
                        println!("error: unknown rhs");
                        return Ok(());
                    }
                },
                _ => {
                    println!("error: unknown lhs");
                    return Ok(());
                }
            },
            Some("<") => match pred.next() {
                Some("#f") => match pred.next().and_then(|n| n.parse::<usize>().ok()) {
                    Some(x) => {
                        while !facets.is_empty() && 2 * facets.len() < x {
                            for cmd in &inst {
                                self.command(cmd.trim().to_owned(), nav, atoms, facets, route, ctx)?
                            }
                        }
                    }
                    _ => {
                        println!("error: unknown rhs");
                        return Ok(());
                    }
                },
                Some("#r") => match pred.next().and_then(|n| n.parse::<usize>().ok()) {
                    Some(x) => {
                        while !facets.is_empty() && route.len() < x {
                            for cmd in &inst {
                                self.command(cmd.trim().to_owned(), nav, atoms, facets, route, ctx)?
                            }
                        }
                    }
                    _ => {
                        println!("error: unknown rhs");
                        return Ok(());
                    }
                },
                _ => {
                    println!("error: unknown lhs");
                    return Ok(());
                }
            },
            Some("<=") => match pred.next() {
                Some("#f") => match pred.next().and_then(|n| n.parse::<usize>().ok()) {
                    Some(x) => {
                        while !facets.is_empty() && 2 * facets.len() <= x {
                            for cmd in &inst {
                                self.command(cmd.trim().to_owned(), nav, atoms, facets, route, ctx)?
                            }
                        }
                    }
                    _ => {
                        println!("error: unknown rhs");
                        return Ok(());
                    }
                },
                Some("#r") => match pred.next().and_then(|n| n.parse::<usize>().ok()) {
                    Some(x) => {
                        while !facets.is_empty() && route.len() <= x {
                            for cmd in &inst {
                                self.command(cmd.trim().to_owned(), nav, atoms, facets, route, ctx)?
                            }
                        }
                    }
                    _ => {
                        println!("error: unknown rhs");
                        return Ok(());
                    }
                },
                _ => {
                    println!("error: unknown lhs");
                    return Ok(());
                }
            },
            _ => {
                println!("error: provide instructions");
                return Ok(());
            }
        };
        Ok(())
    }

    pub fn is_atom(&mut self, nav: &mut Navigator, args: Vec<String>) -> Result<()> {
        match args.get(0).and_then(|a| nav.is_known(a.to_owned())) {
            Some(v) => println!("{v}"),
            _ => println!("error: invalid atom"),
        }
        Ok(())
    }

    pub fn show_atoms(&mut self, nav: &mut Navigator) -> Result<()> {
        nav.atoms().for_each(|a| {
            print!("{a} ");
        });
        println!();
        Ok(())
    }

    pub fn filter_atoms(&mut self, args: Vec<String>, atoms: &mut Vec<String>) -> Result<()> {
        let mut k = 0;
        if let Some(re) = args.get(0).and_then(|s| Regex::new(r#s).ok()) {
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

    pub fn show_program(&mut self, nav: &mut Navigator) -> Result<()> {
        println!("{}", nav.program());
        Ok(())
    }

    pub fn sieve_facets(
        &mut self,
        nav: &mut Navigator,
        facets: &mut Vec<String>,
        args: Vec<String>,
    ) -> Result<()> {
        let fs = if let Some(re) = args.get(0).and_then(|s| Regex::new(r#s).ok()) {
            facets
                .iter()
                .filter(|f| re.is_match(f))
                .cloned()
                .collect::<Vec<_>>()
        } else {
            facets.to_vec()
        };
        nav.sieve(&fs)?;
        Ok(())
    }

    pub fn context(
        &mut self,
        nav: &mut Navigator,
        facets: &mut Vec<String>,
        route: &mut Vec<String>,
        args: Vec<String>,
        ctx: &mut Vec<String>,
    ) -> Result<()> {
        ctx.into_iter()
            .skip(1)
            .for_each(|r| unsafe { nav.remove_rule(r).unwrap_unchecked() });

        ctx.clear();

        match args.get(0) {
            Some(cnf) => {
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

                    nav.add_rule(ic)?;
                }
            }
            _ => (),
        };

        *facets = nav
            .facet_inducing_atoms(route.iter())
            .ok_or(NavigatorError::None)?
            .iter()
            .map(|f| lex::repr(*f))
            .collect();
        Ok(())
    }

    pub fn significance(
        &mut self,
        nav: &mut Navigator,
        route: &mut Vec<String>,
        facets: &mut Vec<String>,
        args: Vec<String>,
    ) -> Result<()> {
        let start = Instant::now();
        let y = args.get(0).unwrap();
        if let Some(re) = args.get(1).and_then(|s| Regex::new(r#s).ok()) {
            nav.significance(&route, y.to_owned(), &facets, re)
        }
        println!("sig time elapsed: {:?}", start.elapsed());
        Ok(())
    }

    pub fn significance_projecting(
        &mut self,
        nav: &mut Navigator,
        facets: &mut Vec<String>,
        atoms: &mut Vec<String>,
        route: &mut Vec<String>,
        args: Vec<String>,
    ) -> Result<()> {
        let y = args.get(0).unwrap();

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
            .symbols()
            .filter(|(s, _)| xs.iter().any(|a| a.starts_with(s)))
            .map(|(s, n)| format!("#show {s}/{n}."))
            .collect::<Vec<_>>()
            .join("\n");

        let s = format!("{shows}\n{or}");

        nav.add_rule(s.clone())?;

        if let Some(re) = args.get(2).and_then(|s| Regex::new(r#s).ok()) {
            nav.significance_projecting(&route, y.to_owned(), &facets, re)
        }

        nav.remove_rule(s.clone())?;
        Ok(())
    }

    pub fn enumerate_projected_solutions(
        &mut self,
        nav: &mut Navigator,
        args: Vec<String>,
        facets: &mut Vec<String>,
        route: &mut Vec<String>,
    ) -> Result<()> {
        let n = nav.enumerate_projected_solutions(
            args.get(0).and_then(|n| n.parse::<usize>().ok()).take(),
            route.iter().chain(args.iter().skip(1)).map(String::as_str),
            facets.clone(),
        )?;
        println!("found {:?}", n);
        Ok(())
    }
    pub fn handle_unknown(&mut self, cmd: &str) -> Result<()> {
        if cmd.starts_with("//") {
            return Ok(());
        }

        println!("noop [unknown command]");
        Ok(())
    }
}