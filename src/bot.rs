use crate::uma::{Uma, StatType, StyleChoice};
use crate::random::{random, get_seed};

pub struct Bot;
impl Bot {
    pub fn new() -> Self { Self }

    pub fn train(&self, uma: &mut Uma, turns: u8) {
        for turn in 0..turns {
            if uma.get_mood().mood_to_value() < 0 {
                let seed = get_seed() + turn as u64 + uma.get_energy() as u64;
                if random(seed, 99) < 10 { uma.recreation(); continue; }
            }
            let all = [StatType::Speed, StatType::Stamina, StatType::Power, StatType::Guts, StatType::Wit];
            let (best_stat, best_fc) = all.iter()
                .map(|s| (*s, uma.failure_chance(s)))
                .min_by_key(|(_, fc)| *fc)
                .unwrap();
            if best_fc > 25 { uma.rest(); } else { uma.train(best_stat, best_fc); }
        }
    }

    pub fn choose_style(&self, uma: &mut Uma) {
        let seed = get_seed() + uma.style.front.index() as u64;
        if random(seed, 99) < 10 {
            let styles = [StyleChoice::Front, StyleChoice::Pace, StyleChoice::Late, StyleChoice::End];
            uma.set_style(styles[random(seed.wrapping_add(1), 3) as usize]);
            return;
        }
        let opts: [(StyleChoice, u16); 4] = [
            (StyleChoice::Front, uma.style.front.index()),
            (StyleChoice::Pace,  uma.style.pace.index()),
            (StyleChoice::Late,  uma.style.late.index()),
            (StyleChoice::End,   uma.style.end.index()),
        ];
        let best_idx = opts.iter().map(|(_, i)| *i).max().unwrap();
        let best: Vec<StyleChoice> = opts.iter().filter(|(_, i)| *i == best_idx).map(|(s,_)| *s).collect();
        uma.set_style(best[random(seed, (best.len() - 1) as u16) as usize]);
    }
}
