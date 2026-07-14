#[pyo3::pymodule]
pub mod wrappers_bindings {

    use crate::cache::FacetCache;
    use crate::modes::{perform_next_step, propose_next_step};
    use pyo3::prelude::*;
    use savan::nav::Navigator;
    use std::num::NonZeroUsize;

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
        pub facet_cache: FacetCache,
        pub program_version: u64,
        pub cache_enabled: bool,
        pub optimized_enabled: bool,
    }

    impl PyNavigator {
        pub(crate) fn invalidate_cache_internal(&mut self) {
            self.program_version = self.program_version.wrapping_add(1);
            self.clear_cache_internal();
        }

        pub(crate) fn clear_cache_internal(&mut self) {
            self.facet_cache.clear();
        }

        pub(crate) fn set_cache_capacity_internal(&mut self, capacity: NonZeroUsize) {
            self.facet_cache.resize(capacity);
        }
    }

    #[pymethods]
    impl PyNavigator {
        #[new]
        pub fn new(source: String, args: Vec<String>) -> PyResult<Self> {
            let nav = Navigator::new(source, args)
                .map_err(|_| pyo3::exceptions::PyRuntimeError::new_err("Navigator::new failed"))?;

            Ok(Self {
                nav,
                facet_cache: FacetCache::new(),
                program_version: 0,
                cache_enabled: false,
                optimized_enabled: false,
            })
        }

        pub fn set_cache_enabled(&mut self, enabled: bool) {
            self.cache_enabled = enabled;
        }

        pub fn is_cache_enabled(&self) -> bool {
            self.cache_enabled
        }

        pub fn clear_cache(&mut self) {
            self.clear_cache_internal();
        }

        pub fn set_cache_capacity(&mut self, capacity: usize) -> PyResult<()> {
            let capacity = NonZeroUsize::new(capacity).ok_or_else(|| {
                pyo3::exceptions::PyValueError::new_err("cache capacity must be greater than zero")
            })?;
            self.set_cache_capacity_internal(capacity);
            Ok(())
        }

        pub fn cache_capacity(&self) -> usize {
            self.facet_cache.capacity()
        }

        pub fn set_optimized_enabled(&mut self, enabled: bool) {
            self.optimized_enabled = enabled;
        }

        pub fn is_optimized_enabled(&self) -> bool {
            self.optimized_enabled
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
}
