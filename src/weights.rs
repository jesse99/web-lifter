use crate::default;
use crate::errors::Error;
use crate::validation_err;
use core::fmt;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::{
    collections::{HashMap, HashSet},
    fmt::Formatter,
};

// TODO Might want to support bumper plates though that would get quite annoying because
// we'd want to use them whenever possible. For example, if we had [15 bumper x8, 20 x6]
// and want 60 lbs we'd normally select 20 x3 (for single plates) but with bumpers we'd
// want 15 bumper x4. Though this is probably more doable now that we're enumerating
// plates.

pub fn format_weight(weight: f32, suffix: &str) -> String {
    let mut s = format!("{weight:.3}");
    while s.ends_with("0") {
        s.remove(s.len() - 1);
    }
    if s.ends_with(".") {
        s.remove(s.len() - 1);
    }
    format!("{s}{suffix}")
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Plate {
    pub weight: f32,
    pub count: i32, // how many of this plate the user has
}

impl Plate {
    pub fn new(weight: f32, count: i32) -> Plate {
        Plate { weight, count }
    }
}

#[derive(Clone, Debug)]
pub struct Weight {
    weight: InternalWeight,
}

impl Weight {
    fn discrete(value: f32) -> Weight {
        Weight {
            weight: InternalWeight::Discrete(value),
        }
    }

    fn error(mesg: String, target: f32) -> Weight {
        Weight {
            weight: InternalWeight::Error(mesg, target),
        }
    }

    fn plates(plates: Plates) -> Weight {
        Weight {
            weight: InternalWeight::Plates(plates),
        }
    }

    /// The actual weight, may include stuff like a bar weight.
    pub fn value(&self) -> f32 {
        match &self.weight {
            InternalWeight::Discrete(v) => *v,
            InternalWeight::Error(_, v) => *v,
            InternalWeight::Plates(p) => p.weight(),
        }
    }

    /// The weight as a string, e.g. "165 lbs".
    pub fn text(&self) -> String {
        match &self.weight {
            InternalWeight::Discrete(v) => format_weight(*v, " lbs"),
            InternalWeight::Error(_, _) => String::new(),
            InternalWeight::Plates(p) => format_weight(p.weight(), " lbs"),
        }
    }

    /// More information about the weight e.g. "45 + 10 + 5" (if plates are being used)
    /// or "40 + 2.5 magnet" (for dumbbells with optional magnets). Note that for
    /// DualPlates this returns the plates for only one side.
    pub fn details(&self) -> Option<String> {
        match &self.weight {
            InternalWeight::Discrete(_) => None,
            InternalWeight::Error(m, _) => Some(m.clone()),
            InternalWeight::Plates(p) => Some(format!("{}", p)),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WeightSet {
    /// Used for stuff like dumbbells and cable machines. Weights should be sorted from
    /// smallest to largest.
    Discrete(Vec<f32>), // TODO: support extra weights, eg magnets for dumbbells, maybe allow user to name them "magnet" or whatever

    /// Used for stuff like barbell exercises and leg presses. Plates are added in pairs.
    /// Includes an optional bar weight. Plates should be sorted from smallest to largest.
    DualPlates(Vec<Plate>, Option<f32>),
}

/// Collections of weight sets that are shared across programs, e.g. there could be sets
/// for dummbells, a cable machine, plates for OHP, and plates for deadlifts.
#[derive(Debug, Serialize, Deserialize)]
pub struct Weights {
    sets: HashMap<String, WeightSet>,

    // All non-duplicate combinations of DualPlates for every weight sorted by smallest
    // weight to largest. Note that these are the plates added to one side of the bar.
    #[serde(default)]
    combos: HashMap<String, Vec<Vec<Plate>>>,
}

impl Weights {
    pub fn new() -> Weights {
        Weights {
            sets: HashMap::new(),
            combos: HashMap::new(),
        }
    }

    pub fn fixup(&mut self) {
        self.rebuild_combos()
    }

    // pub fn names(&self) -> impl Iterator<Item = &String> + '_ {
    //     self.sets.keys()
    // }

    pub fn items(&self) -> impl Iterator<Item = (&String, &WeightSet)> + '_ {
        self.sets.iter()
    }

    pub fn get(&self, name: &str) -> Option<&WeightSet> {
        self.sets.get(name)
    }

    pub fn add(&mut self, name: String, set: WeightSet) {
        if let WeightSet::DualPlates(plates, _) = &set {
            let old = self.combos.insert(name.clone(), enumerate_weights(plates));
            assert!(old.is_none());
        }

        let old = self.sets.insert(name, set);
        assert!(old.is_none());
    }

    /// Used for warmups and backoff sets. May return a weight larger than target.
    pub fn closest(&self, name: &str, target: f32) -> Weight {
        if let Some(set) = self.sets.get(name) {
            match set {
                WeightSet::Discrete(weights) => Weight::discrete(closest_discrete(target, weights)),
                WeightSet::DualPlates(_, bar) => match self.combos.get(name) {
                    Some(enums) => Weight::plates(closest_dual(target, enums, bar)),
                    None => Weight::error(format!("There is no combos named '{name}'"), target),
                },
            }
        } else if name.is_empty() {
            Weight::discrete(target)
        } else {
            Weight::error(format!("There is no weight set named '{name}'"), target)
        }
    }

    /// Used for worksets. Will not return a weight larger than target.
    pub fn lower(&self, name: &str, target: f32) -> Weight {
        if let Some(set) = self.sets.get(name) {
            match set {
                WeightSet::Discrete(weights) => Weight::discrete(find_discrete(target, weights).0),
                WeightSet::DualPlates(_, bar) => match self.combos.get(name) {
                    Some(enums) => Weight::plates(lower_dual(target, enums, bar)),
                    None => Weight::error(format!("There is no combos named '{name}'"), target),
                },
            }
        } else if name.is_empty() {
            Weight::discrete(target)
        } else {
            Weight::error(format!("There is no weight set named '{name}'"), target)
        }
    }

    /// Return the next weight larger than target.
    pub fn advance(&self, name: &str, target: f32) -> Weight {
        if let Some(set) = self.sets.get(name) {
            match set {
                WeightSet::Discrete(weights) => {
                    Weight::discrete(find_discrete(target + 0.001, weights).1)
                }
                WeightSet::DualPlates(_, bar) => match self.combos.get(name) {
                    Some(enums) => Weight::plates(upper_dual(target, enums, bar)),
                    None => Weight::error(format!("There is no combos named '{name}'"), target),
                },
            }
        } else {
            Weight::error(format!("There is no weight set named '{name}'"), target)
        }
    }

    pub fn try_set_discrete_weights(&mut self, sets: Vec<String>) -> Result<(), Error> {
        let valid = |w: &WeightSet| matches!(w, WeightSet::Discrete(_));
        self.validate_set_weight_sets(&sets, valid)?;
        self.do_set_weight_sets(sets, valid, default::default_discrete());
        Ok(())
    }

    pub fn try_set_plate_weights(&mut self, sets: Vec<String>) -> Result<(), Error> {
        let valid = |w: &WeightSet| matches!(w, WeightSet::Discrete(_));
        self.validate_set_weight_sets(&sets, valid)?;
        self.do_set_weight_sets(sets, valid, default::default_plates());
        Ok(())
    }

    pub fn try_change_set(
        &mut self,
        old_name: &str,
        new_name: &str,
        weights: WeightSet,
    ) -> Result<(), Error> {
        self.validate_change_set(old_name, new_name, &weights)?;
        self.do_change_set(old_name, new_name, weights);
        Ok(())
    }

    fn validate_set_weight_sets<F>(&self, sets: &[String], valid: F) -> Result<(), Error>
    where
        F: Fn(&WeightSet) -> bool,
    {
        let mut names = HashSet::new();
        for name in sets.iter() {
            if name.trim().is_empty() {
                return validation_err!("Weight set names cannot be empty.");
            }

            let added = names.insert(name.clone());
            if !added {
                return validation_err!("'{name}' appears more than once.");
            }

            if let Some(old) = self.sets.get(name) {
                if !valid(old) {
                    return validation_err!(
                        "'{name}' is already a weight set with a different type."
                    );
                }
            }
        }
        Ok(())
    }

    fn do_set_weight_sets<F>(&mut self, sets: Vec<String>, valid: F, exemplar: WeightSet)
    where
        F: Fn(&WeightSet) -> bool,
    {
        let mut old_valid = HashMap::new();
        let mut old_invalid = HashMap::new();
        for (name, set) in self.sets.drain() {
            if valid(&set) {
                old_valid.insert(name, set);
            } else {
                old_invalid.insert(name, set);
            }
        }

        // Note that this will implicitly delete sets that are no longer named.
        let mut new_sets = HashMap::new();
        for name in sets.into_iter() {
            if let Some(set) = old_valid.remove(&name) {
                new_sets.insert(name, set);
            } else {
                new_sets.insert(name, exemplar.clone());
            }
        }

        self.sets = new_sets;
        for (name, set) in old_invalid.drain() {
            let old = self.sets.insert(name, set);
            assert!(old.is_none(), "validation should have prevented this");
        }

        self.combos.clear();
        self.rebuild_combos();
    }

    fn validate_change_set(
        &self,
        old_name: &str,
        new_name: &str,
        weights: &WeightSet,
    ) -> Result<(), Error> {
        fn validate_discrete(weights: &[f32]) -> Result<(), Error> {
            if weights.is_empty() {
                return validation_err!("There should be at least one weight.");
            }
            for (i, weight) in weights.iter().enumerate() {
                if *weight < 0.0 {
                    return validation_err!("Weights cannot be negative.");
                } else if *weight == 0.0 {
                    return validation_err!("Weights cannot be zero.");
                } else if i + 1 < weights.len() && (*weight - weights[i + 1]).abs() < 0.001 {
                    return validation_err!("Weights cannot have duplicate values.",);
                } else if i + 1 < weights.len() && *weight > weights[i + 1] {
                    return validation_err!("Weights should be from smaller to larger.",);
                }
            }
            Ok(())
        }

        fn validate_dual(plates: &[Plate], bar: &Option<f32>) -> Result<(), Error> {
            if plates.is_empty() {
                return validation_err!("There should be at least one plate.");
            }
            for (i, plate) in plates.iter().enumerate() {
                if plate.weight < 0.0 {
                    return validation_err!("Plate weights cannot be negative.");
                } else if plate.weight == 0.0 {
                    return validation_err!("Plate weights cannot be zero.");
                } else if i + 1 < plates.len()
                    && (plate.weight - plates[i + 1].weight).abs() < 0.001
                {
                    return validation_err!("Plate weights cannot have duplicate values.",);
                } else if i + 1 < plates.len() && plate.weight > plates[i + 1].weight {
                    return validation_err!("Plate weights should be from smaller to larger.",);
                }
            }
            if let Some(weight) = bar {
                if *weight < 0.0 {
                    return validation_err!("Bar weight cannot be negative.");
                } else if *weight == 0.0 {
                    return validation_err!("Bar weight cannot be zero.");
                }
            }
            Ok(())
        }

        if new_name.trim().is_empty() {
            return validation_err!("The weight set name cannot be empty.");
        } else if new_name == "None" {
            return validation_err!("The weight set name cannot be 'None'.",);
        } else if new_name != old_name && self.get(new_name).is_some() {
            return validation_err!("The new weight set name already exists.",);
        }

        match weights {
            WeightSet::Discrete(weights) => validate_discrete(weights)?,
            WeightSet::DualPlates(plates, bar) => validate_dual(plates, bar)?,
        }

        Ok(())
    }

    fn do_change_set(&mut self, _old_name: &str, new_name: &str, weights: WeightSet) {
        // Might make more sense to remove the old weightset but if we do that we'll
        // also need to change each exercise to use the new name.
        if let WeightSet::DualPlates(plates, _) = &weights {
            self.combos
                .insert(new_name.to_string(), enumerate_weights(plates));
        }
        self.sets.insert(new_name.to_string(), weights);
    }

    fn rebuild_combos(&mut self) {
        for (name, set) in self.sets.iter() {
            if let WeightSet::DualPlates(plates, _) = set {
                if !self.combos.contains_key(name) {
                    self.combos.insert(name.clone(), enumerate_weights(plates));
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
enum InternalWeight {
    Discrete(f32),
    Error(String, f32),
    Plates(Plates),
}

#[derive(Clone)]
struct Plates {
    plates: Vec<Plate>, // largest to smallest
    bar: Option<f32>,
    dual: bool, // if true plates are added two at a time and Display shows one side
}

impl PartialEq for Plate {
    fn eq(&self, other: &Self) -> bool {
        let a = (1000.0 * self.weight) as i32;
        let b = (1000.0 * other.weight) as i32;
        self.count == other.count && a == b
    }
}

impl Eq for Plate {}

impl Plates {
    fn new(plates: Vec<Plate>, bar: Option<f32>, dual: bool) -> Plates {
        let mut plates = Plates { plates, bar, dual };
        plates
            .plates
            .sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());
        plates
    }

    fn weight(&self) -> f32 {
        self.plates
            .iter()
            .fold(0.0, |sum, p| sum + (p.weight * (p.count as f32)))
            + self.bar.unwrap_or(0.0)
    }
}

impl fmt::Display for Plates {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut v = Vec::new();

        let multiplier = if self.dual { 2 } else { 1 };
        for plate in self.plates.iter() {
            if plate.count == multiplier {
                v.push(format_weight(plate.weight, ""));
            } else {
                v.push(format_weight(
                    plate.weight,
                    &format!(" x{}", plate.count / multiplier),
                ));
            }
        }

        write!(f, "{}", v.join(" + "))
    }
}

impl fmt::Debug for Plates {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

fn closest_discrete(target: f32, weights: &[f32]) -> f32 {
    let (lower, upper) = find_discrete(target, weights);
    if target - lower <= upper - target {
        lower
    } else {
        upper
    }
}

fn make_dual(plates: &[Plate]) -> Vec<Plate> {
    plates
        .iter()
        .map(|p| Plate {
            weight: p.weight,
            count: p.count * 2,
        })
        .collect()
}

fn summed_weight(plates: &[Plate]) -> f32 {
    plates
        .iter()
        .fold(0.0, |sum, p| sum + (p.weight * (p.count as f32)))
}

fn closest_dual(target: f32, enums: &[Vec<Plate>], bar: &Option<f32>) -> Plates {
    fn find_best(target: i32, lhs: &[Plate], rhs: &[Plate]) -> Vec<Plate> {
        let l = (1000.0 * summed_weight(lhs)) as i32;
        let r = (1000.0 * summed_weight(rhs)) as i32;
        if i32::abs(target - l) < i32::abs(target - r) {
            make_dual(lhs)
        } else {
            make_dual(rhs)
        }
    }

    let target = target - bar.unwrap_or(0.0);
    let target = target / 2.0; // because this is dual plates
                               // println!("target: {target:.1} (adjusted)");

    let t = (1000.0 * target) as i32;
    let i = enums.binary_search_by(|p| {
        let w = (1000.0 * summed_weight(p)) as i32;
        w.cmp(&t)
    });
    // println!("search: {i:?}");
    match i {
        Ok(i) => Plates::new(make_dual(&enums[i]), *bar, true), // exact match
        Err(i) => {
            if i > 0 {
                let plates = find_best(t, &enums[i - 1], &enums[i]);
                Plates::new(plates, *bar, true)
            } else if !enums.is_empty() {
                let plates = find_best(t, &Vec::new(), &enums[i]);
                Plates::new(plates, *bar, true)
            } else {
                Plates::new(Vec::new(), *bar, true)
            }
        }
    }
}

fn lower_dual(target: f32, enums: &[Vec<Plate>], bar: &Option<f32>) -> Plates {
    let target = target - bar.unwrap_or(0.0);
    let target = target / 2.0; // because this is dual plates
                               // println!("target: {target:.1} (adjusted)");

    let t = (1000.0 * target) as i32;
    let i = enums.binary_search_by(|p| {
        let w = (1000.0 * summed_weight(p)) as i32;
        w.cmp(&t)
    });
    // println!("search: {i:?}");
    match i {
        Ok(i) => Plates::new(make_dual(&enums[i]), *bar, true), // exact match
        Err(i) => {
            if i > 0 {
                Plates::new(make_dual(&enums[i - 1]), *bar, true)
            } else {
                Plates::new(Vec::new(), *bar, true)
            }
        }
    }
}

fn upper_dual(target: f32, enums: &[Vec<Plate>], bar: &Option<f32>) -> Plates {
    // println!("target: {target:.1}");
    let target = target - bar.unwrap_or(0.0);
    let target = target / 2.0; // because this is dual plates
                               // println!("target: {target:.1} (adjusted)");
                               // println!("enums: {enums:?}");

    let t = (1000.0 * target) as i32;
    let i = enums.binary_search_by(|p| {
        let w = (1000.0 * summed_weight(p)) as i32;
        w.cmp(&t)
    });
    // println!("search: {i:?}");
    match i {
        Ok(i) => {
            if i + 1 < enums.len() {
                Plates::new(make_dual(&enums[i + 1]), *bar, true)
            } else {
                Plates::new(make_dual(&enums[i]), *bar, true)
            }
        }
        Err(i) => {
            if target < 0.0 {
                if bar.is_some() || enums.is_empty() {
                    Plates::new(Vec::new(), *bar, true)
                } else {
                    Plates::new(make_dual(&enums[0]), *bar, true)
                }
            } else if !enums.is_empty() {
                if i < enums.len() {
                    Plates::new(make_dual(&enums[i]), *bar, true)
                } else {
                    Plates::new(make_dual(&enums[i - 1]), *bar, true)
                }
            } else {
                Plates::new(Vec::new(), *bar, true)
            }
        }
    }
}

fn find_discrete(target: f32, weights: &[f32]) -> (f32, f32) {
    let mut lower = weights.first().copied().unwrap_or(0.0);
    let mut upper = f32::MAX;

    for &candidate in weights.iter() {
        if candidate > lower && candidate <= target {
            lower = candidate;
        }
        if candidate < upper && candidate >= target {
            upper = candidate;
        }
    }

    (lower, upper)
}

impl fmt::Debug for Plate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} x{}", format_weight(self.weight, ""), self.count)
    }
}

struct IterN {
    n: u64,
    i: usize,
}

impl IterN {
    /// Takes an n representing the number of plates and returns (count, index) tuples
    /// where count is the number of plates at index. Note that this does not verify that
    /// the (count, index) is sane.
    fn new(n: u64) -> Self {
        let i = 0;
        IterN { n, i }
    }
}

impl Iterator for IterN {
    type Item = (i32, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.n > 0 {
            let count = self.n % 10;
            let index = self.i;

            self.n /= 10;
            self.i += 1;

            Some((count as i32, index))
        } else {
            None
        }
    }
}

/// Returns all non-duplicate weight combinations of plates sorted from smallest total
/// weight to largest. TODO probably want to make this a method on Weights
fn enumerate_weights(plates: &[Plate]) -> Vec<Vec<Plate>> {
    enum Status {
        Valid,
        Invalid,
        Overflow,
    }

    // TODO: need to restrict max plate count to 9
    fn is_valid(n: u64, plates: &[Plate]) -> Status {
        for (count, index) in IterN::new(n) {
            if index >= plates.len() {
                return Status::Overflow;
            } else if 2 * count > plates[index].count {
                // 2* should only be done for dual plates
                return Status::Invalid;
            }
        }
        Status::Valid
    }

    fn increment(n: u64, plates: &[Plate]) -> Option<u64> {
        let mut n = n;
        loop {
            n += 1;
            match is_valid(n, plates) {
                Status::Valid => return Some(n),
                Status::Overflow => return None,
                Status::Invalid => (),
            }
        }
    }

    fn get_candidate(n: u64, plates: &[Plate]) -> Vec<Plate> {
        let mut possible = Vec::with_capacity(plates.len());
        for (count, index) in IterN::new(n) {
            assert!(count <= plates[index].count);
            if count > 0 {
                possible.push(Plate::new(plates[index].weight, count));
            }
        }
        possible
    }

    // I expect that there are smarter ways to do this, but:
    // 1) This is very fast even with lots of plate sizes and counts.
    // 2) This will work even for those unfortunates with really weird collections of plates.
    let mut n: u64 = 0; // where n = 2045 means 5 of the largest plate, 4 of the next largest, etc
    let mut candidates: HashMap<i32, Vec<Plate>> = HashMap::new();
    while let Some(new) = increment(n, plates) {
        n = new;
        let candidate = get_candidate(n, plates);

        let weight = candidate
            .iter()
            .fold(0.0, |acc, e| acc + (e.count as f32) * e.weight);
        let candidate_weight = 1000 * (weight as i32); // f32 isn't Hash
        let candidate_count = candidate.iter().fold(0, |acc, e| acc + e.count);
        match candidates.entry(candidate_weight) {
            Entry::Occupied(mut occupied) => {
                // Prefer solutions with the least number of plates.
                let old_count = occupied.get().iter().fold(0, |acc, e| acc + e.count);
                if candidate_count < old_count {
                    occupied.insert(candidate);
                }
            }
            Entry::Vacant(vacant) => _ = vacant.insert(candidate),
        }
    }
    let mut result: Vec<Vec<Plate>> = candidates.drain().map(|(_, plates)| plates).collect();
    result.sort_by(|a, b| {
        // sort so smallest total weights are first
        let a = a
            .iter()
            .fold(0.0, |acc, e| acc + (e.count as f32) * e.weight);
        let b = b
            .iter()
            .fold(0.0, |acc, e| acc + (e.count as f32) * e.weight);
        let a = (1000.0 * a) as i32;
        let b = (1000.0 * b) as i32;
        a.cmp(&b)
    });
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enumerate0() {
        let plates = vec![];
        let v = enumerate_weights(&plates);
        assert_eq!(v.len(), 0);
    }

    #[test]
    fn enumerate1() {
        let plates = vec![Plate::new(45.0, 2)];
        let v = enumerate_weights(&plates);
        println!("{v:?}");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0], vec![Plate::new(45.0, 1)]);
    }

    #[test]
    fn enumerate2a() {
        let plates = vec![Plate::new(45.0, 4)];
        let v = enumerate_weights(&plates);
        println!("{v:?}");
        assert_eq!(v.len(), 2);
        assert_eq!(v[0], vec![Plate::new(45.0, 1)]);
        assert_eq!(v[1], vec![Plate::new(45.0, 2)]);
    }

    #[test]
    fn enumerate2b() {
        let plates = vec![Plate::new(45.0, 2), Plate::new(25.0, 2)];
        let v = enumerate_weights(&plates);
        println!("{v:?}");
        assert_eq!(v.len(), 3);
        assert_eq!(v[0], vec![Plate::new(25.0, 1)]);
        assert_eq!(v[1], vec![Plate::new(45.0, 1)]);
        assert_eq!(v[2], vec![Plate::new(45.0, 1), Plate::new(25.0, 1)]);
    }

    #[test]
    fn enumerate4() {
        let plates = vec![Plate::new(45.0, 4), Plate::new(25.0, 4)];
        let v = enumerate_weights(&plates);
        println!("{v:?}");
        assert_eq!(v.len(), 8);
        assert_eq!(v[0], vec![Plate::new(25.0, 1)]);
        assert_eq!(v[1], vec![Plate::new(45.0, 1)]);
        assert_eq!(v[2], vec![Plate::new(25.0, 2)]);
        assert_eq!(v[3], vec![Plate::new(45.0, 1), Plate::new(25.0, 1)]);
        assert_eq!(v[4], vec![Plate::new(45.0, 2)]);
        assert_eq!(v[5], vec![Plate::new(45.0, 1), Plate::new(25.0, 2)]);
        assert_eq!(v[6], vec![Plate::new(45.0, 2), Plate::new(25.0, 1)]);
        assert_eq!(v[7], vec![Plate::new(45.0, 2), Plate::new(25.0, 2)]);
    }

    #[test]
    fn enumerate_lots() {
        let plates = vec![
            Plate::new(100.0, 4), // 6074 before pruning
            Plate::new(45.0, 4),
            Plate::new(25.0, 2),
            Plate::new(10.0, 2),
            Plate::new(5.0, 2),
            Plate::new(2.5, 2),
            Plate::new(1.25, 2),
        ];
        let v = enumerate_weights(&plates);
        assert_eq!(v.len(), 247);
    }

    #[test]
    fn empty1() {
        // Return something half-way sane if there aren't any weights.
        let mut weights = Weights::new();
        let name = "dumbbells";
        weights.add(name.to_owned(), WeightSet::Discrete(vec![]));
        assert_eq!(weights.closest(name, 10.0).value(), 0.0); // if there are no dumbbells at all then we can't use a weight
        assert_eq!(weights.closest(name, 10.0).text(), "0 lbs");
    }

    #[test]
    fn empty2() {
        // Signal a problem if there isn't a weight set.
        let weights = Weights::new();
        let name = "dumbbells";
        assert_eq!(weights.closest(name, 10.0).value(), 10.0); // if there's not a weight set then we may as well just return target
        assert_eq!(
            weights.closest(name, 10.0).details().unwrap(),
            "There is no weight set named 'dumbbells'"
        );
    }

    #[test]
    fn discrete() {
        let mut weights = Weights::new();
        let name = "dumbbells";
        weights.add(
            name.to_owned(),
            WeightSet::Discrete(vec![5.0, 10.0, 15.0, 20.0]),
        );
        assert_eq!(weights.closest(name, 0.0).value(), 5.0);
        assert_eq!(weights.closest(name, 0.0).text(), "5 lbs");

        assert_eq!(weights.closest(name, 4.0).value(), 5.0);
        assert_eq!(weights.closest(name, 4.0).text(), "5 lbs");

        assert_eq!(weights.closest(name, 5.0).value(), 5.0);
        assert_eq!(weights.closest(name, 5.0).text(), "5 lbs");

        assert_eq!(weights.closest(name, 6.0).value(), 5.0);
        assert_eq!(weights.closest(name, 6.0).text(), "5 lbs");

        assert_eq!(weights.closest(name, 9.0).value(), 10.0);
        assert_eq!(weights.closest(name, 9.0).text(), "10 lbs");

        assert_eq!(weights.closest(name, 18.0).value(), 20.0);
        assert_eq!(weights.closest(name, 18.0).text(), "20 lbs");

        assert_eq!(weights.closest(name, 30.0).value(), 20.0);
        assert_eq!(weights.closest(name, 30.0).text(), "20 lbs");
    }

    fn check2(target: f32, lower: &str, upper: &str, plates: &Plates) {
        println!("-----------------------------------------------------");
        let enums = enumerate_weights(&plates.plates);
        println!("target: {target:.1}");
        println!("plates: {:?}", &plates.plates);
        println!("bar: {:?}", plates.bar);
        println!("enums: {enums:?}");

        println!("-----------------------");
        let l = lower_dual(target, &enums, &plates.bar);
        println!("-----------------------");
        let u = upper_dual(target, &enums, &plates.bar);

        println!("l: {l:?}");
        println!("u: {u:?}");

        assert!(summed_weight(&l.plates) <= target);
        // Note that upper may be < target if run out of weights

        let l = format!("{}", l);
        let u = format!("{}", u);
        assert!(
            l == lower,
            "lower FAILED target: {target} actual: {l:?} expected: {lower:?}"
        );
        assert!(
            u == upper,
            "upper FAILED target: {target} actual: {u:?} expected: {upper:?}"
        );
    }

    #[test]
    fn dual_plates() {
        let plate1 = Plate {
            weight: 5.0,
            count: 6,
        };
        let plate2 = Plate {
            weight: 10.0,
            count: 6,
        };
        let plate3 = Plate {
            weight: 25.0,
            count: 4,
        };
        let plate4 = Plate {
            weight: 45.0,
            count: 4,
        };
        let plates = Plates::new(vec![plate1, plate2, plate3, plate4], None, true);

        check2(11.0, "5", "10", &plates); // on one side
        check2(14.0, "5", "10", &plates);
        check2(18.0, "5", "10", &plates);
        check2(20.0, "10", "10 + 5", &plates);
        check2(21.0, "10", "10 + 5", &plates);
        check2(30.0, "10 + 5", "10 x2", &plates);
        check2(40.0, "10 x2", "25", &plates);
        check2(50.0, "25", "25 + 5", &plates);
        check2(103.0, "25 x2", "45 + 10", &plates);
        check2(120.0, "25 x2 + 10", "45 + 10 x2", &plates);
        check2(130.0, "45 + 10 x2", "45 + 25", &plates);
        check2(135.0, "45 + 10 x2", "45 + 25", &plates);
        check2(160.0, "45 + 25 + 10", "45 + 25 + 10 + 5", &plates);
        check2(205.0, "45 x2 + 10", "45 + 25 x2 + 10", &plates);
        check2(230.0, "45 x2 + 25", "45 x2 + 25 + 5", &plates);
        check2(240.0, "45 x2 + 25 + 5", "45 x2 + 25 + 10", &plates);
        check2(250.0, "45 x2 + 25 + 10", "45 x2 + 25 + 10 + 5", &plates);
        check2(260.0, "45 x2 + 25 + 10 + 5", "45 x2 + 25 + 10 x2", &plates);
        check2(270.0, "45 x2 + 25 + 10 x2", "45 x2 + 25 x2", &plates);
        check2(
            300.0,
            "45 x2 + 25 x2 + 10",
            "45 x2 + 25 x2 + 10 + 5",
            &plates,
        );
        check2(
            320.0,
            "45 x2 + 25 x2 + 10 x2",
            "45 x2 + 25 x2 + 10 x2 + 5",
            &plates,
        );
        check2(
            340.0,
            "45 x2 + 25 x2 + 10 x3",
            "45 x2 + 25 x2 + 10 x3 + 5",
            &plates,
        );
        check2(
            380.0,
            "45 x2 + 25 x2 + 10 x3 + 5 x3",
            "45 x2 + 25 x2 + 10 x3 + 5 x3",
            &plates,
        );
    }

    #[test]
    fn dual_plates_with_bar() {
        let plate1 = Plate {
            // we'll use a somewhat unusual plate distribution here
            weight: 5.0,
            count: 3,
        };
        let plate2 = Plate {
            weight: 10.0,
            count: 2,
        };
        let plate3 = Plate {
            weight: 25.0,
            count: 6,
        };
        let plate4 = Plate {
            weight: 45.0,
            count: 2,
        };
        let plates = Plates::new(vec![plate1, plate2, plate3, plate4], Some(45.0), true);

        check2(60.0, "5", "10", &plates); // can only add a max of 2 5's
        check2(70.0, "10", "10 + 5", &plates);
        check2(80.0, "10 + 5", "25", &plates); // can only add a max of 2 10's
        check2(90.0, "10 + 5", "25", &plates);
        check2(120.0, "25 + 10", "25 + 10 + 5", &plates);
        check2(150.0, "25 x2", "45 + 10", &plates);
        check2(180.0, "25 x2 + 10 + 5", "45 + 25", &plates);
        check2(200.0, "25 x3", "45 + 25 + 10", &plates);
        check2(230.0, "25 x3 + 10 + 5", "45 + 25 x2", &plates);
        check2(260.0, "45 + 25 x2 + 10", "45 + 25 x2 + 10 + 5", &plates);
        check2(290.0, "45 + 25 x3", "45 + 25 x3 + 5", &plates);
        check2(320.0, "45 + 25 x3 + 10 + 5", "45 + 25 x3 + 10 + 5", &plates);
    }

    #[test]
    fn closest_dual_test() {
        fn check(target: f32, expected: &str, plates: &[Plate], bar: Option<f32>) {
            println!("-----------------------------------------------------");
            let enums = enumerate_weights(plates);
            println!("target: {target:.1}");
            println!("plates: {plates:?}");
            println!("bar: {bar:?}");
            println!("enums: {enums:?}");
            let actual = closest_dual(target, &enums, &bar);
            println!("actual: {actual:?}");

            let actual = format!("{}", actual);
            assert!(
                actual == expected,
                "FAILED target: {target} actual: {actual} expected: {expected}"
            );
        }

        let plate1 = Plate {
            weight: 5.0,
            count: 6,
        };
        let plate2 = Plate {
            weight: 10.0,
            count: 6,
        };
        let plate3 = Plate {
            weight: 25.0,
            count: 4,
        };
        let plate4 = Plate {
            weight: 45.0,
            count: 4,
        };
        let plates = vec![plate1, plate2, plate3, plate4];

        check(0.0, "", &plates, None); // degenerate case
        check(4.0, "", &plates, None);
        check(8.0, "5", &plates, None);
        check(0.0, "", &plates, Some(45.0));
        check(40.0, "", &plates, Some(45.0));

        check(92.0, "45", &plates, None); // lower is best
        check(47.0, "", &plates, Some(45.0));
        check(58.0, "5", &plates, Some(45.0)); // 5 == 55, 10 == 65

        check(97.0, "25 x2", &plates, None); // upper is best
        check(63.0, "10", &plates, Some(45.0));
    }

    #[test]
    fn advance_discrete() {
        let mut weights = Weights::new();
        let name = "dumbbells";
        weights.add(
            name.to_owned(),
            WeightSet::Discrete(vec![5.0, 10.0, 15.0, 20.0]),
        );
        assert_eq!(weights.advance(name, 0.0).value(), 5.0);
        assert_eq!(weights.advance(name, 4.0).value(), 5.0);
        assert_eq!(weights.advance(name, 5.0).value(), 10.0);
        assert_eq!(weights.advance(name, 6.0).value(), 10.0);
    }

    #[test]
    fn advance_dual_plates() {
        let plate1 = Plate {
            weight: 5.0,
            count: 6,
        };
        let plate2 = Plate {
            weight: 10.0,
            count: 6,
        };
        let plate3 = Plate {
            weight: 25.0,
            count: 4,
        };
        let plate4 = Plate {
            weight: 45.0,
            count: 4,
        };
        let plates = vec![plate1, plate2, plate3, plate4];
        let mut weights = Weights::new();
        let name = "plates";
        weights.add(name.to_owned(), WeightSet::DualPlates(plates, None));

        assert_eq!(weights.advance(name, 0.0).value(), 10.0);
        assert_eq!(weights.advance(name, 4.0).value(), 10.0);
        assert_eq!(weights.advance(name, 11.0).value(), 20.0);
        assert_eq!(weights.advance(name, 25.0).value(), 30.0);
    }

    #[test]
    fn advance_dual_plates_with_bar() {
        let plate1 = Plate {
            weight: 5.0,
            count: 6,
        };
        let plate2 = Plate {
            weight: 10.0,
            count: 6,
        };
        let plate3 = Plate {
            weight: 25.0,
            count: 4,
        };
        let plate4 = Plate {
            weight: 45.0,
            count: 4,
        };
        let plates = vec![plate1, plate2, plate3, plate4];
        let mut weights = Weights::new();
        let name = "plates";
        weights.add(name.to_owned(), WeightSet::DualPlates(plates, Some(45.0)));

        assert_eq!(weights.advance(name, 0.0).value(), 45.0);
        assert_eq!(weights.advance(name, 45.0).value(), 55.0);
        assert_eq!(weights.advance(name, 50.0).value(), 55.0);
        assert_eq!(weights.advance(name, 55.0).value(), 65.0);
    }
}
