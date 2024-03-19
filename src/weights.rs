use core::fmt;
use std::{collections::HashMap, fmt::Formatter};

// TODO Might want to support bumper plates though that would get quite annoying because
// we'd want to use them whenever possible. For example, if we had [15 bumper x8, 20 x6]
// and want 60 lbs we'd normally select 20 x3 (for single plates) but with bumpers we'd
// want 15 bumper x4.

#[derive(Clone, Copy)]
pub struct Plate {
    pub weight: f32,
    pub count: i32, // how many of this plate the user has
}

impl Plate {
    pub fn new(weight: f32, count: i32) -> Plate {
        Plate { weight, count }
    }
}

#[derive(Clone)]
pub struct Weight {
    weight: InternalWeight,
}

#[derive(Clone)]
enum InternalWeight {
    Discrete(f32),
    Error(String, f32),
    Plates(Plates),
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

#[derive(Clone, Debug)]
pub enum WeightSet {
    /// Used for stuff like dumbbells and cable machines. Weights should be sorted from
    /// smallest to largest.
    Discrete(Vec<f32>), // TODO: support extra weights, eg magnets for dumbbells

    /// Used for stuff like barbell exercises and leg presses. Plates are added in pairs.
    /// Includes an optional bar weight. Plates should be sorted from smallest to largest.
    DualPlates(Vec<Plate>, Option<f32>),
}

/// Collections of weight sets that are shared across programs, e.g. there could be sets
/// for dummbells, a cable machine, and plates for a barbell.
pub struct Weights {
    sets: HashMap<String, WeightSet>,
}

impl Weights {
    pub fn new() -> Weights {
        Weights {
            sets: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: String, set: WeightSet) {
        let old = self.sets.insert(name, set);
        assert!(old.is_none());
    }

    /// Used for warmups and backoff sets. May return a weight larger than target.
    pub fn closest(&self, name: &str, target: f32) -> Weight {
        if let Some(set) = self.sets.get(name) {
            match set {
                WeightSet::Discrete(weights) => Weight::discrete(closest_discrete(target, weights)),
                WeightSet::DualPlates(plates, bar) => {
                    Weight::plates(closest_dual(target, plates, bar))
                }
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
                WeightSet::DualPlates(plates, bar) => {
                    Weight::plates(lower_dual(target, plates, bar))
                }
            }
        } else if name.is_empty() {
            Weight::discrete(target)
        } else {
            Weight::error(format!("There is no weight set named '{name}'"), target)
        }
    }

    /// Return the next weight larger than target.
    pub fn advance(&self, name: &str, target: f32) -> Weight {
        let target = target + 0.001;
        if let Some(set) = self.sets.get(name) {
            match set {
                WeightSet::Discrete(weights) => Weight::discrete(find_discrete(target, weights).1),
                WeightSet::DualPlates(plates, bar) => {
                    Weight::plates(upper_dual(target, plates, bar))
                }
            }
        } else {
            Weight::error(format!("There is no weight set named '{name}'"), target)
        }
    }
}

#[derive(Clone)]
struct Plates {
    plates: Vec<Plate>, // largest to smallest
    bar: Option<f32>,
    dual: bool, // if true plates are added two at a time and Display shows one side
}

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

    fn bar(&self) -> f32 {
        self.bar.unwrap_or(0.0)
    }

    fn smallest(&self) -> Option<&Plate> {
        self.plates.last()
    }

    fn count(&self, weight: f32) -> i32 {
        assert!(weight > 0.0);
        if let Some(index) = self
            .plates
            .iter()
            .position(|p| (p.weight - weight).abs() < 0.001)
        {
            self.plates[index].count
        } else {
            0
        }
    }

    fn add(&mut self, plate: Plate) {
        assert!(plate.weight > 0.0);
        assert!(plate.count > 0);
        assert!(!self.dual || plate.count % 2 == 0);

        if let Some(old) = self
            .plates
            .iter_mut()
            .find(|p| (p.weight - plate.weight).abs() < 0.001)
        {
            old.count += plate.count;
        } else {
            self.plates.push(plate);
            self.plates
                .sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());
        }
    }

    fn remove(&mut self, weight: f32, count: i32) {
        assert!(weight > 0.0);
        assert!(count > 0);
        assert!(!self.dual || count % 2 == 0);

        if let Some(index) = self
            .plates
            .iter_mut()
            .position(|p| (p.weight - weight).abs() < 0.001)
        {
            assert!(self.plates[index].count >= count);

            if self.plates[index].count > count {
                self.plates[index].count -= count;
            } else {
                self.plates.remove(index);
            }
        } else {
            // Not really an error but shouldn't happen so we'll complain in debug.
            assert!(false, "didn't find matching plate");
        }
    }

    // Largest to smallest
    fn iter(&self) -> impl DoubleEndedIterator<Item = &Plate> + '_ {
        self.plates.iter()
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

fn closest_discrete(target: f32, weights: &Vec<f32>) -> f32 {
    let (lower, upper) = find_discrete(target, weights);
    if target - lower <= upper - target {
        lower
    } else {
        upper
    }
}

fn too_small_target(target: f32, plates: &Plates, bar: &Option<f32>) -> Option<Plates> {
    // Degenerate case: target is smaller than smallest weight.
    // println!("target: {target}");
    if let Some(smallest) = plates.smallest() {
        // println!(
        //     "smallest: {} bar: {} sum: {}",
        //     smallest.weight,
        //     plates.bar(),
        //     2.0 * smallest.weight + plates.bar()
        // );
        if target < 2.0 * smallest.weight + plates.bar() {
            // 58 < 2*5 + 45
            // println!("degenerate case");
            let upper = find_dual_upper(target, &plates);
            if plates.bar() > 0.0 && (target - plates.bar()).abs() < (target - upper.weight()).abs()
            {
                return Some(Plates::new(Vec::new(), bar.clone(), plates.dual));
            } else {
                return Some(upper);
            }
        }
    }
    None
}

fn closest_dual(target: f32, plates: &Vec<Plate>, bar: &Option<f32>) -> Plates {
    let plates = Plates::new(plates.clone(), bar.clone(), true);
    if let Some(plates) = too_small_target(target, &plates, bar) {
        plates
    } else {
        let lower = find_dual_lower(target, &plates);
        let upper = find_dual_upper(target, &plates);
        let (l, u) = (lower.weight(), upper.weight());
        if target - l <= u - target {
            lower
        } else {
            upper
        }
    }
}

fn lower_dual(target: f32, plates: &Vec<Plate>, bar: &Option<f32>) -> Plates {
    let plates = Plates::new(plates.clone(), bar.clone(), true);
    if let Some(plates) = too_small_target(target, &plates, bar) {
        plates
    } else {
        find_dual_lower(target, &plates)
    }
}

fn upper_dual(target: f32, plates: &Vec<Plate>, bar: &Option<f32>) -> Plates {
    let plates = Plates::new(plates.clone(), bar.clone(), true);
    if let Some(plates) = too_small_target(target, &plates, bar) {
        plates
    } else {
        find_dual_upper(target, &plates)
    }
}

fn find_dual_lower(target: f32, plates: &Plates) -> Plates {
    fn add_plates(from: &Plate, lower: &mut Plates, target: f32) {
        let mut count = 0;
        loop {
            let new = ((count + 2) as f32) * from.weight;
            if count + 2 > from.count || lower.weight() + new > target {
                break;
            }
            count += 2;
        }
        if count > 0 {
            lower.add(Plate {
                weight: from.weight,
                count,
            });
            // println!("new lower: {lower}");
        }
    }

    let mut lower = Plates::new(Vec::new(), plates.bar.clone(), plates.dual);

    // Add as many plates as possible from largest to smallest.
    for plate in plates.iter() {
        // println!("lower candidate: {plate:?}");
        add_plates(plate, &mut lower, target);
    }
    lower
}

fn find_dual_upper(target: f32, plates: &Plates) -> Plates {
    fn add_large(from: &Plate, remaining: &mut Plates, upper: &mut Plates, target: f32) {
        let mut count = 0;
        loop {
            let new = ((count + 2) as f32) * from.weight;
            if count + 2 > from.count || upper.weight() + new > target {
                break;
            }
            count += 2;
        }
        if count > 0 {
            remaining.remove(from.weight, count);
            upper.add(Plate {
                weight: from.weight,
                count,
            });
            // println!("new large upper: {upper}");
        }
    }

    fn add_small(from: &Plate, remaining: &Plates, upper: &mut Plates) -> bool {
        if from.count >= 2 {
            if upper.count(from.weight) >= 2 && remaining.count(2.0 * from.weight) >= 2 {
                upper.remove(from.weight, 2);
                upper.add(Plate {
                    weight: 2.0 * from.weight,
                    count: 2,
                });
            } else {
                upper.add(Plate {
                    weight: from.weight,
                    count: 2,
                });
            }
            // println!("new large upper: {upper:?}");
            true
        } else {
            false
        }
    }

    let mut upper = Plates::new(Vec::new(), plates.bar.clone(), plates.dual);
    let mut remaining = plates.clone();

    // Add plates as long as the total is under target from largest to smallest.
    for plate in plates.iter() {
        // println!("upper large candidate: {plate:?}");
        add_large(plate, &mut remaining, &mut upper, target);
    }

    // Then add the smallest plate we can to send us over the target.
    if upper.weight() < target || upper.weight() == 0.0 {
        for plate in remaining.iter().rev() {
            // println!("upper small candidate: {plate:?}");
            if add_small(plate, &remaining, &mut upper) {
                break;
            }
        }

        // If we were forced to add a large plate then we may be able to get closer to
        // target by dropping some smaller plates.
        loop {
            if let Some(smallest) = upper.smallest() {
                let weight = upper.weight() - 2.0 * smallest.weight;
                // println!("smallest: {smallest:?} weight: {weight}");
                if weight >= target && target > 0.0 {
                    upper.remove(smallest.weight, 2);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    upper
}

fn find_discrete(target: f32, weights: &Vec<f32>) -> (f32, f32) {
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

fn format_weight(weight: f32, suffix: &str) -> String {
    let mut s = format!("{weight:.3}");
    while s.ends_with("0") {
        s.remove(s.len() - 1);
    }
    if s.ends_with(".") {
        s.remove(s.len() - 1);
    }
    format!("{s}{suffix}")
}

impl fmt::Debug for Plate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} x{}", format_weight(self.weight, ""), self.count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let l = find_dual_lower(target, plates);
        println!("-----------------------------------------------------");
        let u = find_dual_upper(target, plates);

        assert!(l.weight() <= target);
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
        check2(20.0, "10", "10", &plates);
        check2(21.0, "10", "10 + 5", &plates);
        check2(30.0, "10 + 5", "10 + 5", &plates);
        check2(40.0, "10 x2", "10 x2", &plates);
        check2(50.0, "25", "25", &plates);
        check2(103.0, "45 + 5", "45 + 10", &plates);
        check2(120.0, "45 + 10 + 5", "45 + 10 + 5", &plates);
        check2(130.0, "45 + 10 x2", "45 + 10 x2", &plates);
        check2(135.0, "45 + 10 x2", "45 + 10 x2 + 5", &plates);
        check2(160.0, "45 + 25 + 10", "45 + 25 + 10", &plates);
        check2(205.0, "45 x2 + 10", "45 x2 + 10 + 5", &plates);
        check2(230.0, "45 x2 + 25", "45 x2 + 25", &plates);
        check2(240.0, "45 x2 + 25 + 5", "45 x2 + 25 + 5", &plates);
        check2(250.0, "45 x2 + 25 + 10", "45 x2 + 25 + 10", &plates);
        check2(260.0, "45 x2 + 25 + 10 + 5", "45 x2 + 25 + 10 + 5", &plates);
        check2(270.0, "45 x2 + 25 + 10 x2", "45 x2 + 25 + 10 x2", &plates);
        check2(300.0, "45 x2 + 25 x2 + 10", "45 x2 + 25 x2 + 10", &plates);
        check2(
            320.0,
            "45 x2 + 25 x2 + 10 x2",
            "45 x2 + 25 x2 + 10 x2",
            &plates,
        );
        check2(
            340.0,
            "45 x2 + 25 x2 + 10 x3",
            "45 x2 + 25 x2 + 10 x3",
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
        check2(150.0, "45 + 5", "45 + 10", &plates);
        check2(180.0, "45 + 10 + 5", "45 + 25", &plates);
        check2(200.0, "45 + 25 + 5", "45 + 25 + 10", &plates);
        check2(230.0, "45 + 25 + 10 + 5", "45 + 25 x2", &plates);
        check2(260.0, "45 + 25 x2 + 10", "45 + 25 x2 + 10 + 5", &plates);
        check2(290.0, "45 + 25 x3", "45 + 25 x3 + 5", &plates);
        check2(320.0, "45 + 25 x3 + 10 + 5", "45 + 25 x3 + 10 + 5", &plates);
    }

    #[test]
    fn closest_dual_test() {
        fn check(target: f32, expected: &str, plates: &Vec<Plate>, bar: Option<f32>) {
            println!("-----------------------------------------------------");
            let actual = closest_dual(target, plates, &bar);

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

        check(0.0, "5", &plates, None); // degenerate case
        check(4.0, "5", &plates, None);
        check(8.0, "5", &plates, None);
        check(0.0, "", &plates, Some(45.0));
        check(40.0, "", &plates, Some(45.0));

        check(92.0, "45", &plates, None); // lower is best
        check(47.0, "", &plates, Some(45.0));
        check(58.0, "5", &plates, Some(45.0)); // 5 == 55, 10 == 65

        check(97.0, "45 + 5", &plates, None); // upper is best
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
        assert_eq!(weights.advance(name, 50.0).value(), 55.0);
        assert_eq!(weights.advance(name, 55.0).value(), 65.0);
    }
}
