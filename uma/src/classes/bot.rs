use crate::classes::uma::{Uma, StatType, StyleChoice};
use crate::utils::random::{random, get_seed};

pub struct Bot;

impl Bot {
    pub fn new() -> Self {
        Self
    }

    /// Trains the Uma for the given number of turns using a simple strategy:
    /// - If mood < Normal, 10% chance to do recreation instead
    /// - Train the stat with the lowest failure chance
    /// - If the lowest failure chance exceeds 25%, rest instead
    pub fn train(&self, uma: &mut Uma, turns: u8) {
        for turn in 0..turns {
            // 10% chance of recreation if mood is below Normal (mood_to_value < 0)
            if uma.get_mood().mood_to_value() < 0 {
                let seed = get_seed() + turn as u64 + uma.get_energy() as u64;
                if random(seed, 99) < 10 {
                    uma.recreation();
                    continue;
                }
            }

            // Find stat with lowest failure chance
            let all_stats = [
                StatType::Speed,
                StatType::Stamina,
                StatType::Power,
                StatType::Guts,
                StatType::Wit,
            ];

            let (best_stat, best_fc) = all_stats
                .iter()
                .map(|s| (*s, uma.failure_chance(s)))
                .min_by_key(|(_, fc)| *fc)
                .unwrap();

            if best_fc > 25 {
                uma.rest();
            } else {
                uma.train(best_stat, best_fc);
            }
        }
    }

    /// Chooses the best racing style for the Uma.
    /// 10% chance to pick a completely random style instead.
    /// If multiple styles share the highest grade index, picks one randomly.
    pub fn choose_style(&self, uma: &mut Uma) {
        let seed = get_seed() + uma.style.front.index() as u64;

        // 10% chance to pick a random style
        if random(seed, 99) < 10 {
            let all_styles = [StyleChoice::Front, StyleChoice::Pace, StyleChoice::Late, StyleChoice::End];
            let pick = all_styles[random(seed.wrapping_add(1), 3) as usize];
            uma.set_style(pick);
            return;
        }

        let options: [(StyleChoice, u16); 4] = [
            (StyleChoice::Front, uma.style.front.index()),
            (StyleChoice::Pace,  uma.style.pace.index()),
            (StyleChoice::Late,  uma.style.late.index()),
            (StyleChoice::End,   uma.style.end.index()),
        ];
        let best_idx = options.iter().map(|(_, i)| *i).max().unwrap();
        let best: Vec<StyleChoice> = options.iter()
            .filter(|(_, i)| *i == best_idx)
            .map(|(sc, _)| *sc)
            .collect();

        let pick = best[(random(seed, (best.len() - 1) as u16)) as usize];
        uma.set_style(pick);
    }
}
