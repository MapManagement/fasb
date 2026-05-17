use pyo3::prelude::*;
use savan::lex;
use savan::nav::{facets::Facets, Navigator};

#[pyclass]
pub struct PyNavigator {
    nav: std::cell::RefCell<Navigator>,
}

#[pymethods]
impl PyNavigator {
    #[new]
    fn new(source: String, args: Vec<String>) -> PyResult<Self> {
        let nav = Navigator::new(source, args)
            .map_err(|_| pyo3::exceptions::PyRuntimeError::new_err("Navigator::new failed"))?;

        Ok(Self {
            nav: std::cell::RefCell::new(nav),
        })
    }

    fn facet_inducing_atoms(&self, route: Vec<String>) -> PyResult<Vec<String>> {
        let mut nav = self.nav.borrow();

        let facets = nav.facet_inducing_atoms(route.iter()).ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("facet_including_atoms failed")
        })?;

        Ok(facets.iter().map(|f| lex::repr(*f)).collect())
    }
}
