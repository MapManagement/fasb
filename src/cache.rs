use crate::wrappers::wrappers_bindings::PyNavigator;
use lru::LruCache;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use savan::lex;
use savan::nav::facets::Facets;
use std::num::NonZeroUsize;

const DEFAULT_FACET_CACHE_CAPACITY: usize = 128;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum FacetCacheKind {
    Normal,
    Projecting,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FacetCacheKey {
    route: Vec<String>,
    program_version: u64,
    kind: FacetCacheKind,
}

impl FacetCacheKey {
    fn new(route: &[String], program_version: u64, kind: FacetCacheKind) -> Self {
        Self {
            route: route.to_vec(),
            program_version,
            kind,
        }
    }
}

pub struct FacetCache {
    entries: LruCache<FacetCacheKey, Vec<String>>,
    capacity: NonZeroUsize,
}

impl FacetCache {
    pub fn new() -> Self {
        let capacity = NonZeroUsize::new(DEFAULT_FACET_CACHE_CAPACITY).unwrap();

        Self {
            entries: LruCache::new(capacity),
            capacity,
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn capacity(&self) -> usize {
        self.capacity.get()
    }

    pub fn resize(&mut self, capacity: NonZeroUsize) {
        self.entries.resize(capacity);
        self.capacity = capacity;
    }

    fn get(&mut self, key: &FacetCacheKey) -> Option<Vec<String>> {
        self.entries.get(key).cloned()
    }

    fn put(&mut self, key: FacetCacheKey, facets: Vec<String>) {
        self.entries.put(key, facets);
    }
}

impl Default for FacetCache {
    fn default() -> Self {
        Self::new()
    }
}

pub fn cached_facets(nav: &mut PyNavigator, route: &[String]) -> PyResult<Vec<String>> {
    cached_facets_with_kind(
        nav,
        route,
        FacetCacheKind::Normal,
        "facet_inducing_atoms failed",
    )
}

pub fn cached_facets_projecting(nav: &mut PyNavigator, route: &[String]) -> PyResult<Vec<String>> {
    cached_facets_with_kind(
        nav,
        route,
        FacetCacheKind::Projecting,
        "facet_inducing_atoms_projecting failed",
    )
}

fn cached_facets_with_kind(
    nav: &mut PyNavigator,
    route: &[String],
    kind: FacetCacheKind,
    error: &'static str,
) -> PyResult<Vec<String>> {
    let key = FacetCacheKey::new(route, nav.program_version, kind.clone());

    if nav.cache_enabled {
        if let Some(facets) = nav.facet_cache.get(&key) {
            return Ok(facets);
        }
    }

    let facets = match kind {
        FacetCacheKind::Normal => nav.nav.facet_inducing_atoms(route.iter()),
        FacetCacheKind::Projecting => nav.nav.facet_inducing_atoms_projecting(route.iter()),
    }
    .ok_or(PyRuntimeError::new_err(error))?
    .iter()
    .map(|f| lex::repr(*f))
    .collect::<Vec<_>>();

    if nav.cache_enabled {
        nav.facet_cache.put(key, facets.clone());
    }

    Ok(facets)
}
