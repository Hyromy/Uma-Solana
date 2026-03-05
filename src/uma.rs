use anchor_lang::prelude::*;
use crate::random::{random, get_seed, salt};
use crate::racecourse::{DistanceType, Surface, Racecourse};

pub const STAT_LIMIT: u16 = 1200;

pub fn performance(value: u16) -> &'static str {
    match value {
        0..=50    => "G ",
        51..=100  => "G+",
        101..=150 => "F ",
        151..=200 => "F+",
        201..=250 => "E ",
        251..=300 => "E+",
        301..=350 => "D ",
        351..=400 => "D+",
        401..=450 => "C ",
        451..=500 => "C+",
        501..=600 => "B ",
        601..=700 => "B+",
        701..=800 => "A ",
        801..=900 => "A+",
        901..=1000  => "S ",
        1001..=1100 => "S+",
        1101..=1200 => "SS",
        1201..=u16::MAX => "SS+",
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Uma {
    name: String,

    pub stats: Stats,

    pub track: Track,
    pub distance: Distance,
    pub style: Style,
    chosen_style: StyleChoice,

    energy: u8,
    mood: Mood,

    turns: u8,
    wins: u8,
    races: u8,

    turns_to_race: u8,
    is_end: bool,
    is_human: bool,
}
impl Uma {
    const MAX_ENERGY: u8 = 100;
    pub const NAME_MAX_LEN: usize = 32;

    /// Conservative serialized size used to size the on-chain account.
    ///   name (4+32) + stats (5×2) + track (2) + distance (4) + style (4)
    ///   + chosen_style (1) + energy (1) + mood (1) + turns/wins/races/turns_to_race (4)
    ///   + is_end (1) + is_human (1) = 65
    pub const BORSH_SIZE: usize = 65;

    pub fn new(name: String, human: bool) -> Self {
        let name = if name.len() > Self::NAME_MAX_LEN {
            name[..Self::NAME_MAX_LEN].to_string()
        } else {
            name
        };
        let name_salt = salt(name.clone()) + get_seed();

        Self {
            name,
            stats: Stats {
                speed:   TraineeStat::new(name_salt + 0),
                stamina: TraineeStat::new(name_salt + 1),
                power:   TraineeStat::new(name_salt + 2),
                guts:    TraineeStat::new(name_salt + 3),
                wit:     TraineeStat::new(name_salt + 4),
            },
            track:    Track::new(name_salt + 5),
            distance: Distance::new(name_salt + 6),
            style:    Style::new(name_salt + 7),
            chosen_style: StyleChoice::best_of(Style::new(name_salt + 7)),
            mood: Mood::Normal,
            energy: Self::MAX_ENERGY,
            turns: 0,
            wins: 0,
            races: 0,
            turns_to_race: if human { 11 } else { u8::MAX },
            is_end: false,
            is_human: human,
        }
    }

    pub fn new_placeholder() -> Self {
        Self {
            name: String::new(),
            stats: Stats {
                speed:   TraineeStat { value: 0 },
                stamina: TraineeStat { value: 0 },
                power:   TraineeStat { value: 0 },
                guts:    TraineeStat { value: 0 },
                wit:     TraineeStat { value: 0 },
            },
            track:    Track    { turf: Grade::G, dirt: Grade::G },
            distance: Distance { sprint: Grade::G, mile: Grade::G, medium: Grade::G, long: Grade::G },
            style:    Style    { front: Grade::G, pace: Grade::G, late: Grade::G, end: Grade::G },
            chosen_style: StyleChoice::Front,
            mood: Mood::Normal,
            energy: 0,
            turns: 0, wins: 0, races: 0, turns_to_race: 0,
            is_end: true,
            is_human: false,
        }
    }

    pub fn get_name(&self) -> &str { &self.name }
    pub fn is_human(&self) -> bool { self.is_human }

    pub fn get_surface_grade(&self, surface: &Surface) -> &Grade {
        match surface { Surface::Turf => &self.track.turf, Surface::Dirt => &self.track.dirt }
    }

    pub fn get_distance_grade(&self, distance_type: &DistanceType) -> &Grade {
        match distance_type {
            DistanceType::Sprint => &self.distance.sprint,
            DistanceType::Mile   => &self.distance.mile,
            DistanceType::Medium => &self.distance.medium,
            DistanceType::Long   => &self.distance.long,
        }
    }

    pub fn set_style(&mut self, choice: StyleChoice) { self.chosen_style = choice; }
    pub fn get_chosen_style(&self) -> &StyleChoice { &self.chosen_style }

    pub fn get_chosen_style_grade(&self) -> &Grade {
        match self.chosen_style {
            StyleChoice::Front => &self.style.front,
            StyleChoice::Pace  => &self.style.pace,
            StyleChoice::Late  => &self.style.late,
            StyleChoice::End   => &self.style.end,
        }
    }

    pub fn get_points(&self) -> u16 {
        (self.stats.speed.get_value() + self.stats.stamina.get_value()
            + self.stats.power.get_value() + self.stats.guts.get_value()
            + self.stats.wit.get_value()) / 5
    }

    pub fn get_wins(&self)          -> u8    { self.wins }
    pub fn get_races(&self)         -> u8    { self.races }
    pub fn get_turns(&self)         -> u8    { self.turns }
    pub fn get_turns_to_race(&self) -> u8    { self.turns_to_race }
    pub fn get_energy(&self)        -> u8    { self.energy }
    pub fn get_mood(&self)          -> &Mood { &self.mood }
    pub fn get_max_energy(&self)    -> u8    { Self::MAX_ENERGY }
    pub fn get_is_end(&self)        -> bool  { self.is_end }

    pub fn recreation(&mut self) {
        if self.is_end { msg!("The training has already ended."); return; }
        if self.turns_to_race == 0 { msg!("You cant rest now."); return; }
        self.turns += 1;
        self.turns_to_race -= 1;
        self.mood.better();
        if self.is_human { msg!("{} had fun during recreation and feels better!", self.name); }
        self.try_aptitude_boost();
    }

    pub fn race_result(&mut self, position: u8) {
        self.races += 1;
        if position == 1 {
            self.wins += 1;
            msg!("Congratulations! {} won the race!", self.name);
            let seed = self.stats.speed.get_value() as u64
                + self.stats.stamina.get_value() as u64
                + self.stats.power.get_value() as u64
                + self.stats.guts.get_value() as u64
                + self.stats.wit.get_value() as u64
                + self.wins as u64;
            self.turns_to_race = (random(seed, 7) + 8) as u8;
        } else {
            msg!("{} finished in position {}.", self.name, position);
            let retire_chance: u16 = match self.races {
                0 => 0, 1 => 10, 2 => 20, 3 => 30, 4 => 40, _ => 50,
            };
            if retire_chance > 0 && random(get_seed(), 99) < retire_chance {
                msg!("{} has been retired.", self.name);
                self.is_end = true;
                return;
            }
            let seed = self.stats.speed.get_value() as u64
                + self.stats.stamina.get_value() as u64
                + self.stats.power.get_value() as u64
                + self.stats.guts.get_value() as u64
                + self.stats.wit.get_value() as u64
                + self.races as u64;
            self.turns_to_race = random(seed, 20) as u8;
        }
    }

    pub fn calculate_race_score(&self, track: &Racecourse) -> f32 {
        let dist_grade  = self.get_distance_grade(&track.distance_type);
        let surf_grade  = self.get_surface_grade(&track.surface);
        let style_grade = self.get_chosen_style_grade();

        let eff_speed   = self.stats.speed.get_value()   as f32 * Distance::multiplier(dist_grade);
        let eff_power   = self.stats.power.get_value()   as f32 * Track::multiplier(surf_grade);
        let eff_wit     = self.stats.wit.get_value()     as f32 * Style::multiplier(style_grade);
        let eff_stamina = self.stats.stamina.get_value() as f32;
        let eff_guts    = self.stats.guts.get_value()    as f32;

        let (w_early, w_mid, w_late) = match self.chosen_style {
            StyleChoice::Front => (0.40, 0.35, 0.25),
            StyleChoice::Pace  => (0.25, 0.45, 0.30),
            StyleChoice::Late  => (0.15, 0.35, 0.50),
            StyleChoice::End   => (0.10, 0.25, 0.65),
        };

        let early_score = (eff_power * 0.5 + eff_wit   * 0.5) * w_early;
        let mid_score   = (eff_speed * 0.7 + eff_wit   * 0.3) * w_mid;
        let late_score  = (eff_speed * 0.8 + eff_power * 0.2) * w_late;

        let required_stamina = track.distance as f32 / 10.0;
        let stamina_factor = if eff_stamina < required_stamina {
            0.5 + (eff_stamina / required_stamina) * 0.5
        } else { 1.0 };

        let guts_factor = if stamina_factor < 1.0 { 1.0 + (eff_guts / 10000.0) } else { 1.0 };
        let mood_mod    = 1.0 + self.mood.race_performance();

        let variability = (0.10_f32 - (eff_wit / 20000.0)).max(0.01);
        let rng_range   = (variability * 1000.0) as u16;
        let rng_base    = ((1.0 - variability) * 1000.0) as u16;
        let seed        = get_seed() + salt(self.name.clone());
        let rng         = (rng_base + random(seed, rng_range)) as f32 / 1000.0;

        (early_score + mid_score + (late_score * stamina_factor * guts_factor)) * mood_mod * rng
    }

    pub fn apply_stat_penalty(&mut self, factor: f32) {
        self.stats.speed.scale(factor);   self.stats.stamina.scale(factor);
        self.stats.power.scale(factor);   self.stats.guts.scale(factor);
        self.stats.wit.scale(factor);
    }

    pub fn random_mood(&mut self) {
        let base = get_seed() + self.turns as u64 + salt(self.name.clone()) + self.energy as u64;
        let mut step = 0u64;
        loop {
            let seed = base + (self.mood.mood_to_value() as i64 + 100) as u64 + step;
            if random(seed, 99) >= 5 { break; }
            let prev     = self.mood.mood_to_value();
            let dir_seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            if random(dir_seed, 1) == 0 { self.mood.better(); } else { self.mood.worse(); }
            if self.is_human { msg!("{} now has mood {}", self.name, self.mood.to_string()); }
            if self.mood.mood_to_value() == prev { break; }
            step += 200;
        }
    }

    fn set_energy(&mut self, amount: i8) -> i8 {
        let current    = self.energy as i16;
        let new_energy = current + amount as i16;
        if new_energy >= Self::MAX_ENERGY as i16 {
            let real = (Self::MAX_ENERGY as i16 - current) as i8;
            self.energy = Self::MAX_ENERGY; real
        } else if new_energy < 0 {
            let real = -(current as i8);
            self.energy = 0; real
        } else {
            self.energy = new_energy as u8; amount
        }
    }

    pub fn rest(&mut self) -> i8 {
        if self.is_end { msg!("The training has already ended."); return 0; }
        if self.turns_to_race == 0 { msg!("You cant rest now."); return 0; }
        self.turns += 1;
        self.turns_to_race -= 1;
        let seed   = self.turns as u64 + salt(self.name.clone()) + self.energy as u64;
        let amount = (random(seed, 40) + 30) as i8;
        self.random_mood();
        let recovered = self.set_energy(amount);
        if self.is_human { msg!("{} recovered {} energy.", self.name, recovered); }
        self.try_aptitude_boost();
        recovered
    }

    pub fn failure_chance(&self, stat: &StatType) -> u8 {
        let stat_value = match stat {
            StatType::Speed   => self.stats.speed.get_value(),
            StatType::Stamina => self.stats.stamina.get_value(),
            StatType::Power   => self.stats.power.get_value(),
            StatType::Guts    => self.stats.guts.get_value(),
            StatType::Wit     => self.stats.wit.get_value(),
        };
        stat.failure_chance(self.energy, self.turns as u64, salt(self.name.clone()), stat_value)
    }

    pub fn train(&mut self, stat: StatType, failure_chance: u8) -> u8 {
        if self.is_end { msg!("The training has already ended."); return 0; }
        if self.turns_to_race == 0 { msg!("You cant train now."); return 0; }
        self.random_mood();
        let stat_salt = match stat {
            StatType::Speed=>10u64, StatType::Stamina=>20, StatType::Power=>30,
            StatType::Guts=>40,     StatType::Wit=>50,
        };
        self.turns += 1;
        self.turns_to_race -= 1;
        let roll_seed = self.turns as u64 + self.energy as u64 + salt(self.name.clone()) + stat_salt;
        let roll = random(roll_seed, 99) as u8;
        if failure_chance > 0 && roll < failure_chance {
            if self.is_human { msg!("Training failed! Turn lost."); }
            return 0;
        }
        self.set_energy(stat.energy_cost(salt(self.name.clone()), self.turns, self.energy));
        let bonus  = self.mood.training_bonus();
        let gained = match stat {
            StatType::Speed   => self.stats.speed.train(bonus),
            StatType::Stamina => self.stats.stamina.train(bonus),
            StatType::Power   => self.stats.power.train(bonus),
            StatType::Guts    => self.stats.guts.train(bonus),
            StatType::Wit     => self.stats.wit.train(bonus),
        };
        self.try_aptitude_boost();
        gained
    }

    fn try_aptitude_boost(&mut self) {
        let seed = get_seed() + self.turns as u64 + salt(self.name.clone()) + self.energy as u64 + 7777;
        if random(seed, 99) != 0 { return; }
        let slot  = random(seed.wrapping_add(1), 8);
        let label = match slot {
            0 => { self.track.turf      = self.track.turf.upgrade();      "Turf" }
            1 => { self.track.dirt      = self.track.dirt.upgrade();      "Dirt" }
            2 => { self.distance.sprint = self.distance.sprint.upgrade(); "Sprint" }
            3 => { self.distance.mile   = self.distance.mile.upgrade();   "Mile" }
            4 => { self.distance.medium = self.distance.medium.upgrade(); "Medium" }
            5 => { self.distance.long   = self.distance.long.upgrade();   "Long" }
            6 => { self.style.front     = self.style.front.upgrade();     "Front" }
            7 => { self.style.pace      = self.style.pace.upgrade();      "Pace" }
            8 => { self.style.late      = self.style.late.upgrade();      "Late" }
            _ => { self.style.end       = self.style.end.upgrade();       "End" }
        };
        if self.is_human { msg!("A spark of talent! {} aptitude improved!", label); }
    }
}

// ---------------------------------------------------------------------------

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Stats {
    pub speed: TraineeStat, pub stamina: TraineeStat,
    pub power: TraineeStat, pub guts:    TraineeStat, pub wit: TraineeStat,
}

#[derive(Copy, Clone)]
pub enum StatType { Speed, Stamina, Power, Guts, Wit }
impl StatType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v { 0=>Some(Self::Speed), 1=>Some(Self::Stamina), 2=>Some(Self::Power),
                  3=>Some(Self::Guts),  4=>Some(Self::Wit), _=>None }
    }

    pub fn energy_cost(&self, name_salt: u64, turns: u8, energy: u8) -> i8 {
        let (base, span, stat_salt) = match self {
            Self::Speed   => (-30i8, 15u16, 10u64),
            Self::Stamina => (-30,   15,    20),
            Self::Power   => (-30,   15,    30),
            Self::Guts    => (-30,   15,    40),
            Self::Wit     => ( 10,    5,    50),
        };
        let seed  = name_salt + turns as u64 + energy as u64 + stat_salt;
        base + random(seed, span) as i8
    }

    pub fn failure_chance(&self, energy: u8, turns: u64, name_salt: u64, stat_value: u16) -> u8 {
        let threshold = match self { Self::Wit => 33u8, _ => 50 };
        let base = if energy >= threshold { return 0; }
                   else { ((threshold - energy) * 2).min(100) };
        let stat_salt = match self {
            Self::Speed=>10u64, Self::Stamina=>20, Self::Power=>30, Self::Guts=>40, Self::Wit=>50,
        };
        let reduction = random(turns + name_salt + stat_value as u64 + stat_salt, 25) as u8;
        base.saturating_sub(reduction)
    }

    pub fn to_string(&self) -> &str {
        match self { Self::Speed=>"Speed", Self::Stamina=>"Stamina", Self::Power=>"Power",
                     Self::Guts=>"Guts",   Self::Wit=>"Wit" }
    }
}

// ---------------------------------------------------------------------------

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TraineeStat { pub value: u16 }
impl TraineeStat {
    const LIMIT: u16 = STAT_LIMIT;

    pub fn new(s: u64) -> Self { Self { value: random(get_seed() + s, 30) + 70 } }

    pub fn train(&mut self, mood_bonus: f32) -> u8 {
        let base   = random(get_seed() + self.value as u64, 40) + 10;
        let amount = (base as f32 * (1.0 + mood_bonus)).round() as u16;
        if self.value + amount >= Self::LIMIT {
            let real = Self::LIMIT - self.value;
            self.value = Self::LIMIT; real as u8
        } else { self.value += amount; amount as u8 }
    }

    pub fn get_value(&self) -> u16 { self.value }
    pub fn scale(&mut self, f: f32) { self.value = ((self.value as f32) * f).floor() as u16; }
    pub fn to_string(&self) -> String {
        format!("{} ({} / {})", performance(self.value), self.value, Self::LIMIT)
    }
}

// ---------------------------------------------------------------------------

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum Mood { Great, Good, Normal, Bad, Awful }
impl Mood {
    pub fn to_string(&self) -> &str {
        match self { Mood::Great=>"Great (^)", Mood::Good=>"Good (/)", Mood::Normal=>"Normal (-)",
                     Mood::Bad=>"Bad (\\)",    Mood::Awful=>"Awful (v)" }
    }
    pub fn mood_to_value(&self) -> i8 {
        match self { Mood::Great=>2, Mood::Good=>1, Mood::Normal=>0, Mood::Bad=>-1, Mood::Awful=>-2 }
    }
    pub fn value_to_mood(v: i8) -> Self {
        match v { 2=>Mood::Great, 1=>Mood::Good, 0=>Mood::Normal, -1=>Mood::Bad, _=>Mood::Awful }
    }
    pub fn better(&mut self) -> &Self { *self = Self::value_to_mood((self.mood_to_value()+1).min(2)); self }
    pub fn worse(&mut self)  -> &Self { *self = Self::value_to_mood((self.mood_to_value()-1).max(-2)); self }
    pub fn training_bonus(&self) -> f32  { self.mood_to_value() as f32 * 0.1 }
    pub fn race_performance(&self) -> f32 { self.mood_to_value() as f32 * 0.02 }
}

// ---------------------------------------------------------------------------

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone)]
pub enum Grade { S, A, B, C, D, E, F, G }
impl Grade {
    pub fn to_string(&self) -> &str {
        match self { Grade::S=>"S", Grade::A=>"A", Grade::B=>"B", Grade::C=>"C",
                     Grade::D=>"D", Grade::E=>"E", Grade::F=>"F", Grade::G=>"G" }
    }
    pub fn upgrade(&self) -> Self {
        match self { Grade::S=>Grade::S, Grade::A=>Grade::S, Grade::B=>Grade::A,
                     Grade::C=>Grade::B, Grade::D=>Grade::C, Grade::E=>Grade::D,
                     Grade::F=>Grade::E, Grade::G=>Grade::F }
    }
    pub fn index(&self) -> u16 {
        match self { Grade::G=>0, Grade::F=>1, Grade::E=>2, Grade::D=>3,
                     Grade::C=>4, Grade::B=>5, Grade::A=>6, Grade::S=>7 }
    }
    pub fn from_index(i: u16) -> Self {
        match i { 0=>Grade::G, 1=>Grade::F, 2=>Grade::E, 3=>Grade::D,
                  4=>Grade::C, 5=>Grade::B, 6=>Grade::A, _=>Grade::S }
    }
    pub fn spread(seed: u64, n: usize, best: usize) -> Vec<Self> {
        const B: u16 = 6;
        (0..n).map(|pos| {
            let dist    = (pos as i32 - best as i32).unsigned_abs() as u16;
            let min_idx = B.saturating_sub(dist);
            Grade::from_index(min_idx + random(seed + (pos as u64 + 1) * 10, B - min_idx))
        }).collect()
    }
}

// ---------------------------------------------------------------------------

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Track { pub turf: Grade, pub dirt: Grade }
impl Track {
    pub fn new(seed: u64) -> Self {
        let best  = random(seed, 1) as usize;
        let g     = Grade::spread(seed, 2, best);
        let pen   = random(seed.wrapping_add(101), 5) as u16;
        let (turf, dirt) = if g[0].index() >= g[1].index() {
            (g[0], Grade::from_index(g[1].index().saturating_sub(pen)))
        } else {
            (Grade::from_index(g[0].index().saturating_sub(pen)), g[1])
        };
        Self { turf, dirt }
    }
    pub fn multiplier(g: &Grade) -> f32 {
        const T: [f32;8] = [0.10,0.20,0.40,0.70,0.80,0.90,1.00,1.02];
        T[g.index() as usize]
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Distance { pub sprint: Grade, pub mile: Grade, pub medium: Grade, pub long: Grade }
impl Distance {
    pub fn new(seed: u64) -> Self {
        let best = random(seed, 3) as usize;
        let g    = Grade::spread(seed, 4, best);
        Self { sprint: g[0], mile: g[1], medium: g[2], long: g[3] }
    }
    pub fn multiplier(g: &Grade) -> f32 {
        const T: [f32;8] = [0.10,0.20,0.40,0.60,0.80,0.90,1.00,1.05];
        T[g.index() as usize]
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Style { pub front: Grade, pub pace: Grade, pub late: Grade, pub end: Grade }

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone)]
pub enum StyleChoice { Front, Pace, Late, End }
impl StyleChoice {
    pub fn to_string(&self) -> &str {
        match self { Self::Front=>"Front", Self::Pace=>"Pace", Self::Late=>"Late", Self::End=>"End" }
    }
    pub fn best_of(s: Style) -> Self {
        let opts = [(Self::Front,s.front.index()),(Self::Pace,s.pace.index()),
                    (Self::Late,s.late.index()),(Self::End,s.end.index())];
        opts.iter().max_by_key(|(_,i)| i).unwrap().0
    }
}
impl Style {
    pub fn new(seed: u64) -> Self {
        let best = random(seed, 3) as usize;
        let g    = Grade::spread(seed, 4, best);
        Self { front: g[0], pace: g[1], late: g[2], end: g[3] }
    }
    pub fn multiplier(g: &Grade) -> f32 {
        const T: [f32;8] = [0.05,0.10,0.20,0.40,0.60,0.80,1.00,1.05];
        T[g.index() as usize]
    }
}
