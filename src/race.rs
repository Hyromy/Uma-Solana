use anchor_lang::prelude::*;
use crate::uma::Uma;
use crate::racecourse::Racecourse;
use crate::bot::Bot;
use crate::random::{random, get_seed, random_uma_name};

pub fn prepare_to_race(uma: Uma) -> Result<Race, Uma> {
    if uma.get_turns_to_race() > 0 {
        msg!("You can't race yet! {} turns to go.", uma.get_turns_to_race());
        return Err(uma);
    }

    let bot   = Bot::new();
    let turns = uma.get_turns();
    let races = uma.get_races();

    let (turn_offset, penalty): (i16, f32) = match races {
        0 => (-5, 0.75), 1 => (-4, 0.80), 2 => (-3, 0.85),
        3 => (-2, 0.90), 4 => (-1, 0.95), _ => (0, 1.00),
    };
    let bot_turns = (turns as i16 + turn_offset).max(0) as u8;

    let total_runners: usize = if races == 0 { 8 }
        else { (random(get_seed() + races as u64, 10) + 8) as usize };

    let bot_seeds = [
        "these","strings","are","just","used","to","generate","different","bot","names",
        "so","i","write","a","bunch","of","random","words","here","to","fill","up","the","array",
        "","...","are","you","here","?","this","is","the","end","of","the","line",
        "nahh","just","kidding","one","more","for","good","measure","last","one","promise",
        "ok","bye","for","real","now","see","you","later","alligator",
        "in","a","while","crocodile","after","a","while","crocodile","six","seven",
    ];

    let runners: Vec<Uma> = (0..total_runners - 1).map(|i| {
        let mut r = Uma::new(random_uma_name(bot_seeds[i % bot_seeds.len()].to_string()), false);
        bot.train(&mut r, bot_turns);
        if penalty < 1.0 { r.apply_stat_penalty(penalty); }
        bot.choose_style(&mut r);
        r
    }).collect();

    let mut all = vec![uma];
    all.extend(runners);
    Ok(Race::new(Racecourse::new(), all))
}

pub struct Race { track: Racecourse, runners: Vec<Uma> }
impl Race {
    pub fn new(track: Racecourse, runners: Vec<Uma>) -> Self {
        assert!(runners.len() >= 2);
        let mut race = Self { track, runners };
        race.shuffle();
        race
    }

    fn shuffle(&mut self) {
        let n = self.runners.len();
        let base = get_seed();
        for i in (1..n).rev() {
            let j = random(base + i as u64, i as u16) as usize;
            self.runners.swap(i, j);
        }
    }

    pub fn get_track(&self)           -> &Racecourse  { &self.track }
    pub fn get_runners(&self)         -> &Vec<Uma>     { &self.runners }
    pub fn get_runners_mut(&mut self) -> &mut Vec<Uma> { &mut self.runners }

    pub fn run(&mut self) -> u8 {
        let track = &self.track;
        let mut scores: Vec<(usize, f32)> = self.runners.iter().enumerate()
            .map(|(i, r)| (i, r.calculate_race_score(track)))
            .collect();
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        for (pos, (idx, _)) in scores.iter().enumerate() {
            let r = &self.runners[*idx];
            let tag = if r.is_human() { " <--" } else { "" };
            msg!("  {}. {}{}", pos + 1, r.get_name(), tag);
        }

        let human_idx = self.runners.iter().position(|r| r.is_human()).unwrap();
        scores.iter().position(|(i, _)| *i == human_idx).unwrap() as u8 + 1
    }

    pub fn into_runners(self) -> Vec<Uma> { self.runners }
}
