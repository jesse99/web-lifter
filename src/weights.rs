use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub struct Plate {
    pub weight: f32,
    pub count: i32,   // how many of this plate the user has
    pub bumper: bool, // set for bumper plates
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
    pub fn closest(&self, name: &str, target: f32) -> f32 {
        if let Some(set) = self.sets.get(name) {
            match set {
                WeightSet::Discrete(weights) => closest_discrete(target, weights),
                WeightSet::DualPlates(plates, _) => dual_weight(&closest_dual(target, plates)),
            }
        } else {
            0.0
        }
    }

    /// For Discrete weight sets this will return an empty string. For other sets using
    /// plates this will return stuff like "45 + 10 + 2.5". Note that for DualPlates this
    /// returns the plates for only one side.
    pub fn closest_label(&self, name: &str, target: f32) -> String {
        if let Some(set) = self.sets.get(name) {
            match set {
                WeightSet::Discrete(weights) => {
                    let weight = closest_discrete(target, weights);
                    format_weight(weight, " lbs")
                }
                WeightSet::DualPlates(plates, _) => {
                    let plates = closest_dual(target, plates);
                    format_plates(&plates)
                }
            }
        } else {
            format!("There is no weight set named '{name}'")
        }
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

fn closest_dual(target: f32, plates: &Vec<Plate>) -> Vec<Plate> {
    // Degenerate case: target is smaller than smallest weight.
    println!("target: {target} =============================================");
    if let Some(smallest) = plates.first() {
        if target < 2.0 * smallest.weight {
            println!("degenerate case");
            return find_dual_upper(target, &plates);
        }
    }
    let lower = find_dual_lower(target, plates);
    let upper = find_dual_upper(target, plates);
    let (l, u) = (dual_weight(&lower), dual_weight(&upper));
    if target - l <= u - target {
        lower
    } else {
        upper
    }
}

fn find_dual_lower(target: f32, plates: &Vec<Plate>) -> Vec<Plate> {
    fn add_plates(from: &Plate, lower: &mut Vec<Plate>, target: f32) {
        let mut count = 0;
        loop {
            let new = ((count + 2) as f32) * from.weight;
            if count + 2 > from.count || dual_weight(lower) + new > target {
                break;
            }
            count += 2;
        }
        if count > 0 {
            lower.push(Plate {
                weight: from.weight,
                count,
                bumper: from.bumper,
            });
            println!("new lower: {lower:?}");
        }
    }

    let mut lower = vec![];

    // Add as many plates as possible from largest to smallest.
    for plate in plates.iter().rev() {
        println!("lower candidate: {plate:?}");
        add_plates(plate, &mut lower, target);
    }
    lower
}

fn find_dual_upper(target: f32, plates: &Vec<Plate>) -> Vec<Plate> {
    fn add_large(from: &mut Plate, upper: &mut Vec<Plate>, target: f32) {
        let mut count = 0;
        loop {
            let new = ((count + 2) as f32) * from.weight;
            if count + 2 > from.count || dual_weight(upper) + new > target {
                break;
            }
            count += 2;
        }
        if count > 0 {
            from.count -= count;
            upper.push(Plate {
                weight: from.weight,
                count,
                bumper: from.bumper,
            });
            println!("new large upper: {upper:?}");
        }
    }

    fn add_small(from: &Plate, upper: &mut Vec<Plate>) -> bool {
        if from.count >= 2 {
            if let Some(last) = upper.last_mut() {
                if (last.weight - from.weight).abs() < 0.001 {
                    last.count += 2;
                    println!("updated large upper: {upper:?}");
                    return true;
                }
            }
            upper.push(Plate {
                weight: from.weight,
                count: 2,
                bumper: from.bumper,
            });
            println!("new large upper: {upper:?}");
            true
        } else {
            false
        }
    }

    let mut upper = vec![];
    let mut remaining = plates.clone();

    // Add plates as long as the total is under target from largest to smallest.
    for plate in remaining.iter_mut().rev() {
        println!("upper large candidate: {plate:?}");
        add_large(plate, &mut upper, target);
    }

    // Then add the smallest plate we can to send us over the target.
    for plate in remaining.iter() {
        println!("upper small candidate: {plate:?}");
        if add_small(plate, &mut upper) {
            break;
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

fn dual_weight(plates: &Vec<Plate>) -> f32 {
    plates
        .iter()
        .fold(0.0, |sum, p| sum + (p.weight * (p.count as f32)))
}

fn format_plates(plates: &Vec<Plate>) -> String {
    let mut v = Vec::new();

    for plate in plates {
        if plate.count == 2 {
            v.push(format_weight(plate.weight, ""));
        } else {
            v.push(format_weight(
                plate.weight,
                &format!(" x{}", plate.count / 2),
            ));
        }
    }

    v.join(" + ")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty1() {
        // Return something half-way sane if there aren't any weights.
        let mut weights = Weights::new();
        let name = "dumbbells";
        weights.add(name.to_owned(), WeightSet::Discrete(vec![]));
        assert_eq!(weights.closest(name, 10.0), 0.0);
        assert_eq!(weights.closest_label(name, 10.0), "0 lbs");
    }

    #[test]
    fn empty2() {
        // Signal a problem if there isn't a wei9ght set.
        let weights = Weights::new();
        let name = "dumbbells";
        assert_eq!(weights.closest(name, 10.0), 0.0);
        assert_eq!(
            weights.closest_label(name, 10.0),
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
        assert_eq!(weights.closest(name, 0.0), 5.0);
        assert_eq!(weights.closest_label(name, 0.0), "5 lbs");

        assert_eq!(weights.closest(name, 4.0), 5.0);
        assert_eq!(weights.closest_label(name, 4.0), "5 lbs");

        assert_eq!(weights.closest(name, 5.0), 5.0);
        assert_eq!(weights.closest_label(name, 5.0), "5 lbs");

        assert_eq!(weights.closest(name, 6.0), 5.0);
        assert_eq!(weights.closest_label(name, 6.0), "5 lbs");

        assert_eq!(weights.closest(name, 9.0), 10.0);
        assert_eq!(weights.closest_label(name, 9.0), "10 lbs");

        assert_eq!(weights.closest(name, 18.0), 20.0);
        assert_eq!(weights.closest_label(name, 18.0), "20 lbs");

        assert_eq!(weights.closest(name, 30.0), 20.0);
        assert_eq!(weights.closest_label(name, 30.0), "20 lbs");
    }

    #[test]
    fn dual_plates() {
        // TODO change this to test find_dual directly
        let mut weights = Weights::new();
        let name = "barbell";
        let plate1 = Plate {
            weight: 5.0,
            count: 6,
            bumper: false,
        };
        let plate2 = Plate {
            weight: 10.0,
            count: 6,
            bumper: false,
        };
        let plate3 = Plate {
            weight: 25.0,
            count: 4,
            bumper: false,
        };
        let plate4 = Plate {
            weight: 45.0,
            count: 4,
            bumper: false,
        };
        weights.add(
            name.to_owned(),
            WeightSet::DualPlates(vec![plate1, plate2, plate3, plate4], None),
        );
        assert_eq!(weights.closest(name, 0.0), 10.0);
        assert_eq!(weights.closest_label(name, 0.0), "5"); // on one side

        assert_eq!(weights.closest(name, 9.0), 10.0);
        assert_eq!(weights.closest_label(name, 9.0), "5");

        assert_eq!(weights.closest(name, 18.0), 20.0);
        assert_eq!(weights.closest_label(name, 18.0), "5 x2");

        assert_eq!(weights.closest(name, 30.0), 30.0);
        assert_eq!(weights.closest_label(name, 30.0), "10 + 5");

        assert_eq!(weights.closest(name, 40.0), 40.0);
        assert_eq!(weights.closest_label(name, 40.0), "10 x2");

        assert_eq!(weights.closest(name, 50.0), 50.0);
        assert_eq!(weights.closest_label(name, 50.0), "25");

        assert_eq!(weights.closest(name, 60.0), 60.0);
        assert_eq!(weights.closest_label(name, 60.0), "25 + 5");

        assert_eq!(weights.closest(name, 70.0), 70.0);
        assert_eq!(weights.closest_label(name, 70.0), "25 + 10");

        assert_eq!(weights.closest(name, 100.0), 100.0);
        assert_eq!(weights.closest_label(name, 100.0), "45 + 5");

        assert_eq!(weights.closest(name, 200.0), 200.0);
        assert_eq!(weights.closest_label(name, 200.0), "45 x2 + 10");

        assert_eq!(weights.closest(name, 300.0), 300.0);
        assert_eq!(weights.closest_label(name, 200.0), "45 x2 + 10");

        assert_eq!(weights.closest(name, 350.0), 350.0);
        assert_eq!(
            weights.closest_label(name, 350.0),
            "45 x2 + 25 x2 + 10 x3 + 5"
        );

        assert_eq!(weights.closest(name, 400.0), 370.0);
        assert_eq!(
            weights.closest_label(name, 400.0),
            "45 x2 + 25 x2 + 10 x3 + 5 x3"
        );
    }

    // TODO test barweight
    // TODO test bumpers (prefer bumpers when they are available)
}
