#[pyo3::pymodule]
pub mod wrappers_bindings {

    use crate::modes::{perform_next_step, propose_next_step};
    use pyo3::prelude::*;
    use savan::nav::Navigator;

    use crate::modes::Mode;

    #[pyclass]
    pub struct ModeOptionUsize {
        pub inner: Mode<Option<usize>>,
    }

    #[pymethods]
    impl ModeOptionUsize {
        #[new]
        fn new() -> PyResult<Self> {
            let mode = Mode::GoalOriented(None::<usize>);

            Ok(Self { inner: mode })
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

    pub fn propose_next_step_option_usize(
        mode: &mut ModeOptionUsize,
        nav: &mut PyNavigator,
        mut active: Vec<String>,
        facets: Vec<String>,
    ) -> PyResult<(Vec<String>, Option<(String, Option<usize>)>)> {
        let result = propose_next_step(&mut mode.inner, &mut nav.nav, &mut active, &facets);

        Ok((active, result))
    }

    pub fn perform_next_step_option_usize(
        mode: &mut ModeOptionUsize,
        nav: &mut PyNavigator,
        mut active: Vec<String>,
        facets: Vec<String>,
    ) -> PyResult<(Vec<String>, Option<(String, Option<usize>)>)> {
        let result = perform_next_step(&mut mode.inner, &mut nav.nav, &mut active, &facets);

        Ok((active, result))
    }

    #[pymodule]
    pub fn py_wrappers(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_class::<ModeOptionUsize>()?;
        m.add_class::<PyNavigator>()?;
        Ok(())
    }
}
