use crate::*;
use std::collections::HashMap;

pub struct Notes {
    // TODO will need a (per user) edited table that overrides the default table
    table: HashMap<FormalName, String>,
}

impl Notes {
    pub fn new() -> Notes {
        let mut notes = Notes {
            table: HashMap::new(),
        };

        notes.add(
            "Standing Quad Stretch",
            vec![
                "Stand at a right angle to a wall.",
                "Extend your arm straight out and use the wall for support.",
                "With the other hand grab your ankle and pull your foot back to your butt.",
                "Don't arch or twist your back.",
                "Keep your knees together.",
            ],
            vec![(
                "Link",
                "http://www.exrx.net/Stretches/Quadriceps/Standing.html",
            )],
        );
        notes.add(
            "Side Lying Abduction",
            vec!["Lay down on your side with a forearm supporting your head.",
            "Keeping both legs straight raise your free leg into the air.",
            "Stop lifting once you begin to feel tension in your hips.",
            "Go slowly and keep your back straight."],
            vec![("Link", "https://www.verywellfit.com/side-lying-hip-abductions-techniques-benefits-variations-4783963")],
        );
        notes.add(
            "High bar Squat",
            vec!["Bar goes at the top of shoulders at the base of the neck.",
            "Brace core and unrack the bar.",
            "Toes slightly pointed outward.",
            "Push hips back slightly, chest forward, and squat down.",
            "Keep bar over the middle of your feet.",
            "High bar depth is typically greater than low bar depth.",
            "If your neck gets sore the bar is in the wrong position."],
            vec![("Link", "https://squatuniversity.com/2016/03/18/how-to-perfect-the-high-bar-back-squat-2/"), ("Video", "https://www.youtube.com/watch?v=lUGpa_Wz2gs")],
        );

        notes
    }

    pub fn add(&mut self, name: &str, lines: Vec<&str>, links: Vec<(&str, &str)>) {
        let name = FormalName(name.to_owned());

        let mut text = String::new();
        for line in lines {
            // TODO need to use markup
            text += line;
            text += "\n";
        }
        for (name, link) in links {
            text += name;
            text += ": ";
            text += link;
            text += "\n";
        }

        let old = self.table.insert(name, text);
        assert!(old.is_none());
    }
}
