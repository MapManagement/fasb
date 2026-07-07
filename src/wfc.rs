use savan::nav::{Navigator, facets::Facets};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct WeightedFacet {
    facet: String,
    inclusive: bool,
    weight: f32,
}

pub fn weighted_facet_count(
    nav: &mut Navigator,
    route: &[String],
    weighted_fs: &[WeightedFacet],
) -> Option<f32> {
    let mut score = 0.0;

    // Collect consequences into hash sets so the membership tests below are O(1)
    // instead of O(|bc|)/O(|cc|) linear scans (the `for x in cc` loop was O(|cc|·|bc|)).
    let bc = nav
        .brave_consequences(route.iter())
        .map(|xs| xs.iter().map(|s| s.to_string()).collect::<HashSet<_>>())?;

    if !bc.is_empty() {
        for x in weighted_fs.iter().filter(|w| !w.inclusive) {
            if !bc.contains(&x.facet) {
                score += x.weight
            }
        }
    } else {
        // unsat
        return Some(score);
    }

    let cc = nav
        .cautious_consequences(route.iter())
        .map(|xs| xs.iter().map(|s| s.to_string()).collect::<HashSet<_>>())?;

    for x in weighted_fs.iter().filter(|w| w.inclusive) {
        if cc.contains(&x.facet) {
            score += x.weight
        }
    }

    for x in &cc {
        if !bc.contains(x) {
            score += 1.0
        }
    }

    Some(score)
}

pub fn parse_weighted_facets_from_file(filename: &str) -> Option<Vec<WeightedFacet>> {
    let mut wfcs = vec![];

    for l in std::fs::read_to_string(filename).ok()?.lines() {
        let mut xs = l.split_whitespace();
        let facet = xs.next().map(|s| s.to_string())?;
        let inclusive = xs.next().map(|s| s != "0")?;
        let weight = xs.next().and_then(|s| s.parse::<f32>().ok())?;

        wfcs.push(WeightedFacet {
            facet,
            inclusive,
            weight,
        });
    }

    Some(wfcs)
}
