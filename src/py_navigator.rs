use pyo3::prelude::*;
use savan::nav::Navigator;

use crate::modes::{perform_next_step, propose_next_step, Mode, Step};

#[pyfunction]
pub fn propose_next_step_option_usize(
    mode: &mut ModeOptionUsize,
    nav: &mut PyNavigator,
    mut active: Vec<String>,
    facets: Vec<String>,
) -> PyResult<(Vec<String>, Option<(String, Option<usize>)>)> {
    let result = propose_next_step(&mut mode.inner, &mut nav.nav, &mut active, &facets);

    Ok((active, result))
}

#[pyfunction]
pub fn perform_next_step_option_usize(
    mode: &mut ModeOptionUsize,
    nav: &mut PyNavigator,
    mut active: Vec<String>,
    facets: Vec<String>,
) -> PyResult<(Vec<String>, Option<(String, Option<usize>)>)> {
    let result = perform_next_step(&mut mode.inner, &mut nav.nav, &mut active, &facets);

    Ok((active, result))
}

#[pyclass]
pub struct ModeOptionUsize {
    pub inner: Mode<Option<usize>>,
}

#[pymethods]
impl ModeOptionUsize {
    #[new]
    fn new() -> Self {
        Self {
            inner: Mode::GoalOriented(None::<usize>),
        }
    }

    fn update(&mut self, with: Option<usize>) {
        self.inner.update(with);
    }

    fn propose_facet(
        &mut self,
        nav: &mut PyNavigator,
        mut active: Vec<String>,
        among: Vec<String>,
    ) -> Option<(String, Option<usize>)> {
        self.inner.propose_facet(&mut nav.nav, &mut active, &among)
    }
}

#[pyclass(unsendable)]
pub struct PyNavigator {
    pub nav: Navigator,
}

#[pymethods]
impl PyNavigator {
    #[new]
    pub fn new(source: String, args: Vec<String>) -> PyResult<Self> {
        let nav = Navigator::new(source, args)
            .map_err(|_| pyo3::exceptions::PyRuntimeError::new_err("Navigator::new failed"))?;

        Ok(Self { nav })
    }
}
