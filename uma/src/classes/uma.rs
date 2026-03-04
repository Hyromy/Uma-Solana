use crate::utils::random::{random, get_seed, salt};
use crate::classes::racecourse::{DistanceType, Surface, Racecourse};

/// Hard cap for any trainable stat.
/// The stored value of a TraineeStat can NEVER exceed this during training.
/// Race multipliers (Distance, Track, Style) may produce effective values above
/// this limit, but those are temporary f32 calculations only — never written back.
pub const STAT_LIMIT: u16 = 1200;

pub fn performance(value: u16) -> &'static str {
    match value {
        0..=50 => "G ",
        51..=100 => "G+",
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
        901..=1000 => "S ",
        1001..=1100 => "S+",
        1101..=1200 => "SS",
        1201..=u16::MAX => "SS+",
    }
}

/// An Umamusume (horse girl) that can be trained over multiple turns.
/// Stats and aptitudes are seeded from her name combined with the creation time;
/// energy starts at MAX_ENERGY and mood starts at Normal.
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

    /**
     * Create your Umamusume with a given name.
     * 
     * The initial stats are between 70 and 100.
     */
    pub fn new(name: String, human: bool) -> Self {
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

    /**
     * Get the name of your Umamusume.
     */
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn is_human(&self) -> bool {
        self.is_human
    }

    /// Returns the player's grade for the given track surface.
    pub fn get_surface_grade(&self, surface: &Surface) -> &Grade {
        match surface {
            Surface::Turf => &self.track.turf,
            Surface::Dirt => &self.track.dirt,
        }
    }

    /// Returns the player's grade for the given distance type.
    pub fn get_distance_grade(&self, distance_type: &DistanceType) -> &Grade {
        match distance_type {
            DistanceType::Sprint => &self.distance.sprint,
            DistanceType::Mile   => &self.distance.mile,
            DistanceType::Medium => &self.distance.medium,
            DistanceType::Long   => &self.distance.long,
        }
    }

    /// Set the racing style the Uma will use in the next race.
    pub fn set_style(&mut self, choice: StyleChoice) {
        self.chosen_style = choice;
    }

    /// Get the currently chosen racing style.
    pub fn get_chosen_style(&self) -> &StyleChoice {
        &self.chosen_style
    }

    /// Get the Grade of the currently chosen style.
    pub fn get_chosen_style_grade(&self) -> &Grade {
        match self.chosen_style {
            StyleChoice::Front => &self.style.front,
            StyleChoice::Pace  => &self.style.pace,
            StyleChoice::Late  => &self.style.late,
            StyleChoice::End   => &self.style.end,
        }
    }

    /**
     * Get the performance points of your Umamusume.
     */
    pub fn get_points(&self) -> u16 {
        (
            self.stats.speed.get_value() +
            self.stats.stamina.get_value() +
            self.stats.power.get_value() +
            self.stats.guts.get_value() +
            self.stats.wit.get_value()
        ) / 5
    }

    /**
     * Get the number of wins of your Umamusume.
     */
    pub fn get_wins(&self) -> u8 {
        self.wins
    }

    /// Get the number of races this Uma has participated in.
    pub fn get_races(&self) -> u8 {
        self.races
    }

    /**
     * Get the number of played turns of your Umamusume.
     */
    pub fn get_turns(&self) -> u8 {
        self.turns
    }

    /**
     * Get the number of turns left until the race.
     */
    pub fn get_turns_to_race(&self) -> u8 {
        self.turns_to_race
    }

    /**
     * Get the current energy of your Umamusume.
     */
    pub fn get_energy(&self) -> u8 {
        self.energy
    }

    /**
     * Get the current mood of your Umamusume.
     */
    pub fn get_mood(&self) -> &Mood {
        &self.mood
    }

    /**
     * Get the maximum energy of your Umamusume.
     */
    pub fn get_max_energy(&self) -> u8 {
        Self::MAX_ENERGY
    }

    pub fn get_is_end(&self) -> bool {
        self.is_end
    }

    /**
     * Take a turn to rest, improving mood. Cannot be done on race turns.
     */
    pub fn recreation(&mut self) {
        if self.is_end {
            println!("The training has already ended. No more turns can be taken.");
            return;
        }

        if self.turns_to_race <= 0 {
            println!("You cant rest now.");
            return;
        }

        self.turns += 1;
        self.turns_to_race -= 1;

        self.mood.better();
        if self.is_human { println!("{} had fun during recreation and feels better!", self.name); }

        self.try_aptitude_boost();
    }

    /**
     * Participate in a race
     */
    pub fn race_result(&mut self, position: u8) {
        self.races += 1;
        if position == 1 {
            self.wins += 1;
            println!("Congratulations! {} won the race!", self.name);

            // Calculate next training period: 8–15 turns, seeded from stats + wins
            let seed = self.stats.speed.get_value() as u64
                + self.stats.stamina.get_value() as u64
                + self.stats.power.get_value() as u64
                + self.stats.guts.get_value() as u64
                + self.stats.wit.get_value() as u64
                + self.wins as u64;
            self.turns_to_race = (random(seed, 7) + 8) as u8;

        } else {
            println!("{} finished in position {}.", self.name, position);

            // Retirement chance scales with races (0→0%, 1→10%, ..., 5+→50%)
            let retire_chance: u16 = match self.races {
                0 => 0,
                1 => 10,
                2 => 20,
                3 => 30,
                4 => 40,
                _ => 50,
            };
            if retire_chance > 0 && random(get_seed(), 99) < retire_chance {
                println!("{} has been retired.", self.name);
                self.is_end = true;
                return;
            }

            // Not retired: assign a new training period (8–15 turns)
            let seed = self.stats.speed.get_value() as u64
                + self.stats.stamina.get_value() as u64
                + self.stats.power.get_value() as u64
                + self.stats.guts.get_value() as u64
                + self.stats.wit.get_value() as u64
                + self.races as u64;
            self.turns_to_race = (random(seed, 20) + 0) as u8;
        }
    }

    /**
     * Calculate the race score for this Uma on a given track.
     *
     * Divides the race into three phases (Early, Mid, Late) weighted by the chosen style:
     *   Front: heavy early, fades late.
     *   Pace:  balanced, peaks mid.
     *   Late:  slow start, strong finish.
     *   End:   minimal early, dominant late sprint.
     *
     * Aptitude multipliers (Distance→Speed, Surface→Power, Style→Wit) scale effective stats.
     * Stamina is checked against distance: if insufficient, Late phase is penalized up to 50%.
     * Guts provides a small recovery factor when stamina runs out.
     * Mood shifts the final score by ±4%.
     * Wit reduces RNG variability (higher wit = more consistent result).
     */
    pub fn calculate_race_score(&self, track: &Racecourse) -> f32 {
        let dist_grade  = self.get_distance_grade(&track.distance_type);
        let surf_grade  = self.get_surface_grade(&track.surface);
        let style_grade = self.get_chosen_style_grade();

        let eff_speed   = self.stats.speed.get_value()   as f32 * Distance::multiplier(dist_grade);
        let eff_power   = self.stats.power.get_value()   as f32 * Track::multiplier(surf_grade);
        let eff_wit     = self.stats.wit.get_value()     as f32 * Style::multiplier(style_grade);
        let eff_stamina = self.stats.stamina.get_value() as f32;
        let eff_guts    = self.stats.guts.get_value()    as f32;

        // Phase weights by style
        let (w_early, w_mid, w_late) = match self.chosen_style {
            StyleChoice::Front => (0.40, 0.35, 0.25),
            StyleChoice::Pace  => (0.25, 0.45, 0.30),
            StyleChoice::Late  => (0.15, 0.35, 0.50),
            StyleChoice::End   => (0.10, 0.25, 0.65),
        };

        let early_score = (eff_power * 0.5 + eff_wit   * 0.5) * w_early;
        let mid_score   = (eff_speed * 0.7 + eff_wit   * 0.3) * w_mid;
        let late_score  = (eff_speed * 0.8 + eff_power * 0.2) * w_late;

        // Stamina: penalize late phase if insufficient for the distance
        let required_stamina = track.distance as f32 / 10.0;
        let stamina_factor = if eff_stamina < required_stamina {
            0.5 + (eff_stamina / required_stamina) * 0.5
        } else {
            1.0
        };

        // Guts: small recovery when stamina is lacking
        let guts_factor = if stamina_factor < 1.0 {
            1.0 + (eff_guts / 10000.0)
        } else {
            1.0
        };

        // Mood: ±4% global modifier
        let mood_mod = 1.0 + self.mood.race_performance();

        // RNG variability shrinks as wit grows (floor at ±1%)
        let variability = (0.10_f32 - (eff_wit / 20000.0)).max(0.01);
        let rng_range   = (variability * 1000.0) as u16;
        let rng_base    = ((1.0 - variability) * 1000.0) as u16;
        let seed        = get_seed() + salt(self.name.clone());
        let rng         = (rng_base + random(seed, rng_range)) as f32 / 1000.0;

        (early_score + mid_score + (late_score * stamina_factor * guts_factor)) * mood_mod * rng
    }

    /// Apply a multiplier to all five stats. Used after bot training to add a handicap.
    pub fn apply_stat_penalty(&mut self, factor: f32) {
        self.stats.speed.scale(factor);
        self.stats.stamina.scale(factor);
        self.stats.power.scale(factor);
        self.stats.guts.scale(factor);
        self.stats.wit.scale(factor);
    }

    /**
     * Randomly change the mood of the Uma, with a 5% chance of changing it.
     * If it changes, there is a 50% chance of improving or worsening it.
     * Repeats while the mood keeps changing, with a unique seed per iteration.
     */
    pub fn random_mood(&mut self) {
        let base = get_seed() + self.turns as u64 + salt(self.name.clone()) + self.energy as u64;
        let mut step = 0u64;
        loop {
            let seed = base + (self.mood.mood_to_value() as i64 + 100) as u64 + step;
            if random(seed, 99) >= 5 { break; }

            let prev = self.mood.mood_to_value();
            let dir_seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            if random(dir_seed, 1) == 0 { self.mood.better(); } else { self.mood.worse(); }
            if self.is_human { println!("{} now has mood {}", self.name, self.mood.to_string()); }

            if self.mood.mood_to_value() == prev { break; }
            step += 200;
        }
    }

    /**
     * Modify the energy of your Umamusume by a given amount, ensuring it stays within 0 and MAX_ENERGY.
     */
    fn set_energy(&mut self, amount: i8) -> i8 {
        let current = self.energy as i16;
        let new_energy = current + amount as i16;

        if new_energy >= Self::MAX_ENERGY as i16 {
            let real_amount = (Self::MAX_ENERGY as i16 - current) as i8;
            self.energy = Self::MAX_ENERGY;
            real_amount

        } else if new_energy < 0 {
            let real_amount = -(current as i8);
            self.energy = 0;
            real_amount

        } else {
            self.energy = new_energy as u8;
            amount
        }
    }

    /**
     * Lost a turn and restores energy by a random amount between 30 and 70.
     * 
     * The amount of energy restored is determined by a hash of the current turn, the name, and the current energy
     */
    pub fn rest(&mut self) -> i8 {
        if self.is_end {
            println!("The training has already ended. No more turns can be taken.");
            return 0;
        }

        if self.turns_to_race <= 0 {
            println!("You cant rest now.");
            return 0;
        }

        self.turns += 1;
        self.turns_to_race -= 1;

        let seed = self.turns as u64 + salt(self.name.clone()) + self.energy as u64;
        let amount = (random(seed, 40) + 30) as i8;

        self.random_mood();

        let recovered = self.set_energy(amount);
        if self.is_human { println!("{} recovered {} energy.", self.name, recovered); }

        self.try_aptitude_boost();
        recovered
    }

    /**
     * Calculate the failure chance for training a given stat based on the current energy, turns, name, and stat value.
     * The failure chance is higher when energy is low
     */
    pub fn failure_chance(&self, stat: &StatType) -> u8 {
        let stat_value = match stat {
            StatType::Speed =>   self.stats.speed.get_value(),
            StatType::Stamina => self.stats.stamina.get_value(),
            StatType::Power =>   self.stats.power.get_value(),
            StatType::Guts =>    self.stats.guts.get_value(),
            StatType::Wit =>     self.stats.wit.get_value(),
        };
        stat.failure_chance(self.energy, self.turns as u64, salt(self.name.clone()), stat_value)
    }

    /**
     * Lost a turn and train a given stat, which will increase the stat by a random amount between 10 and 50, but has a chance to fail if energy is low.
     * The failure chance must be calculated beforehand using `failure_chance`, and passed here.
     * If the training fails, the stat does not increase, energy doesn't change and the turn is lost.
     */
    pub fn train(&mut self, stat: StatType, failure_chance: u8) -> u8 {
        if self.is_end {
            println!("The training has already ended. No more turns can be taken.");
            return 0;
        }

        if self.turns_to_race <= 0 {
            println!("You cant train now.");
            return 0;
        }

        self.random_mood();

        let stat_salt = match stat {
            StatType::Speed =>   10_u64,
            StatType::Stamina => 20_u64,
            StatType::Power =>   30_u64,
            StatType::Guts =>    40_u64,
            StatType::Wit =>     50_u64,
        };
        self.turns += 1;
        self.turns_to_race -= 1;
        let roll_seed = self.turns as u64
            + self.energy as u64
            + salt(self.name.clone())
            + stat_salt;
        let roll = random(roll_seed, 99) as u8;
        if failure_chance > 0 && roll < failure_chance {
            if self.is_human { println!("Training failed! Turn lost."); }
            return 0;
        }

        self.set_energy(
            stat.energy_cost(
                salt(self.name.clone()),
                self.turns,
                self.energy
            )
        );

        let bonus = self.mood.training_bonus();
        let gained = match stat {
            StatType::Speed =>   self.stats.speed.train(bonus),
            StatType::Stamina => self.stats.stamina.train(bonus),
            StatType::Power =>   self.stats.power.train(bonus),
            StatType::Guts =>    self.stats.guts.train(bonus),
            StatType::Wit =>     self.stats.wit.train(bonus),
        };

        self.try_aptitude_boost();
        gained
    }

    /// 1% chance per action (train/rest/recreation) of upgrading a random
    /// terrain-surface, distance, or racing-style aptitude by one grade.
    fn try_aptitude_boost(&mut self) {
        let seed = get_seed()
            + self.turns as u64
            + salt(self.name.clone())
            + self.energy as u64
            + 7777_u64; // unique offset so this roll doesn't alias other randoms

        if random(seed, 99) != 0 {
            return; // ~99% of the time, nothing happens
        }

        // Pick one of 9 aptitude slots uniformly
        let slot = random(seed.wrapping_add(1), 8);
        let label = match slot {
            0 => { self.track.turf      = self.track.turf.upgrade();         "Turf" }
            1 => { self.track.dirt      = self.track.dirt.upgrade();         "Dirt" }
            2 => { self.distance.sprint = self.distance.sprint.upgrade();    "Sprint" }
            3 => { self.distance.mile   = self.distance.mile.upgrade();      "Mile" }
            4 => { self.distance.medium = self.distance.medium.upgrade();    "Medium" }
            5 => { self.distance.long   = self.distance.long.upgrade();      "Long" }
            6 => { self.style.front     = self.style.front.upgrade();        "Front" }
            7 => { self.style.pace      = self.style.pace.upgrade();         "Pace" }
            8 => { self.style.late      = self.style.late.upgrade();         "Late" }
            _ => { self.style.end       = self.style.end.upgrade();          "End" }
        };

        if self.is_human {
            println!("A spark of talent! {} aptitude improved!", label);
        }
    }
}

/// The five trainable stats of an Uma.
pub struct Stats {
    pub speed: TraineeStat,
    pub stamina: TraineeStat,
    pub power: TraineeStat,
    pub guts: TraineeStat,
    pub wit: TraineeStat,
}

/// Identifies which of the five trainable stats is being referenced or trained.
#[derive(Copy, Clone)]
pub enum StatType {
    Speed,
    Stamina,
    Power,
    Guts,
    Wit,
}
impl StatType {
    /**
     * Calculate the energy cost for training this stat, based on the Uma's name, current turn and current energy.
     * the cost is between -30 and -15 for stand stats, and between 10 and 15 for lazy stats.
     */
    fn energy_cost(&self, name_salt: u64, turns: u8, energy: u8) -> i8 {
        let (base, span, stat_salt) = match self {
            StatType::Speed =>   (-30_i8, 15, 10_u64),
            StatType::Stamina => (-30_i8, 15, 20_u64),
            StatType::Power =>   (-30_i8, 15, 30_u64),
            StatType::Guts =>    (-30_i8, 15, 40_u64),
            StatType::Wit =>     ( 10_i8,  5, 50_u64),
        };

        let seed = name_salt + turns as u64 + energy as u64 + stat_salt;
        let bonus = random(seed, span) as i8;
        base + bonus
    }

    /**
     * Calculate the failure chance for training this stat, based on the Uma's current energy, turns, name and stat value.
     * All stats start failing when energy drops below 50.
     * Wit is an exception: it starts failing below 33 energy, making it safer to train at low energy.
     */
    pub fn failure_chance(&self, energy: u8, turns: u64, name_salt: u64, stat_value: u16) -> u8 {
        let threshold = match self {
            StatType::Wit => 33_u8,
            _             => 50_u8,
        };

        let base = if energy >= threshold {
            return 0;
        } else {
            let diff = threshold - energy;
            (diff * 2).min(100) as u8
        };

        let stat_salt = match self {
            StatType::Speed =>   10_u64,
            StatType::Stamina => 20_u64,
            StatType::Power =>   30_u64,
            StatType::Guts =>    40_u64,
            StatType::Wit =>     50_u64,
        };
        let seed = turns + name_salt + stat_value as u64 + stat_salt;
        let reduction = random(seed, 25) as u8;

        base.saturating_sub(reduction)
    }

    pub fn to_string(&self) -> &str {
        match self {
            StatType::Speed => "Speed",
            StatType::Stamina => "Stamina",
            StatType::Power => "Power",
            StatType::Guts => "Guts",
            StatType::Wit => "Wit",
        }
    }
}

/// A single trainable stat. Its stored value is always within 0..=STAT_LIMIT.
pub struct TraineeStat {
    value: u16,
}
impl TraineeStat {
    const LIMIT: u16 = STAT_LIMIT;

    /**
     * Create a new TraineeStat with a random initial value between 70 and 100, based on the given salt and the current time.
     */
    pub fn new(salt: u64) -> Self {
        let seed = get_seed() + salt as u64;
        Self { value: random(seed, 30) + 70}
    }

    /**
     * Train this stat, increasing its value by a random amount between 10 and 50 (modified by mood bonus),
     * but not exceeding the limit of 1200.
     * mood_bonus comes from Mood::training_bonus(): -0.2 (Awful) to +0.2 (Great).
     *
     * INVARIANT: self.value is always <= STAT_LIMIT after this call.
     * Effective values above STAT_LIMIT only exist during race calculations.
     */
    pub fn train(&mut self, mood_bonus: f32) -> u8 {
        let seed = get_seed() + self.value as u64;
        let base = random(seed, 40) + 10;
        let amount = (base as f32 * (1.0 + mood_bonus)).round() as u16;

        if self.value + amount >= Self::LIMIT {
            let real_amount = Self::LIMIT - self.value;
            self.value = Self::LIMIT;
            real_amount as u8
        
        } else {
            self.value += amount;
            amount as u8
        }
    }

    /**
     * Get the current value of this stat.
     */
    pub fn get_value(&self) -> u16 {
        self.value
    }

    /// Multiply this stat's value by `factor`, rounded down. Used for bot penalties.
    pub fn scale(&mut self, factor: f32) {
        self.value = ((self.value as f32) * factor).floor() as u16;
    }

    /**
     * Get a string representation of this stat, showing both the grade and the numeric value.
     */
    pub fn to_string(&self) -> String {
        format!("{} ({} / {})", performance(self.value), self.value, TraineeStat::LIMIT)
    }
}

/// The current emotional state of the Uma.
/// Mood shifts randomly each turn and affects training gains and race performance.
pub enum Mood {
    Great,
    Good,
    Normal,
    Bad,
    Awful,
}
impl Mood {
    /**
     * Returns a human-readable label for the mood, e.g. "Great (^)".
     */
    pub fn to_string(&self) -> &str {
        match self {
            Mood::Great => "Great (^)",
            Mood::Good => "Good (/)",
            Mood::Normal => "Normal (-)",
            Mood::Bad => "Bad (\\)",
            Mood::Awful => "Awful (v)",
        }
    }

    /**
     * Maps the mood to an integer in -2..=2 (Awful=-2, Great=2).
     */
    pub fn mood_to_value(&self) -> i8 {
        match self {
            Mood::Great =>  2,
            Mood::Good =>   1,
            Mood::Normal => 0,
            Mood::Bad =>   -1,
            Mood::Awful => -2,
        }
    }

    /**
     * Constructs a Mood from an integer in -2..=2. Panics on out-of-range values.
     */
    pub fn value_to_mood(value: i8) -> Self {
        match value {
            2 => Mood::Great,
            1 => Mood::Good,
            0 => Mood::Normal,
            -1 => Mood::Bad,
            -2 => Mood::Awful,
            _ => panic!("Invalid mood value"),
        }
    }

    /**
     * Improves the mood by one step, capped at Great.
     */
    pub fn better(&mut self) -> &Self {
        *self = Self::value_to_mood((self.mood_to_value() + 1).min(2));
        self
    }

    /**
     * Worsens the mood by one step, floored at Awful.
     */
    pub fn worse(&mut self) -> &Self {
        *self = Self::value_to_mood((self.mood_to_value() - 1).max(-2));
        self
    }

    /**
     * Training stat multiplier: -0.2 (Awful) to +0.2 (Great).
     * Applied as: amount = base * (1.0 + training_bonus())
     */
    pub fn training_bonus(&self) -> f32 {
        self.mood_to_value() as f32 * 0.1
    }

    /**
     * Race performance multiplier: -0.04 (Awful) to +0.04 (Great).
     * Applied as first global multiplier before Distance/Track/Style.
     */
    pub fn race_performance(&self) -> f32 {
        self.mood_to_value() as f32 * 0.02
    }
}

/// Aptitude grade from G (worst) to S (best), used for Track, Distance and Style.
/// Each grade maps to a stat multiplier defined in the corresponding struct.
#[derive(Copy, Clone)]
pub enum Grade {
    S,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}
impl Grade {
    /**
     * Returns the single-letter name of the grade.
     */
    pub fn to_string(&self) -> &str {
        match self {
            Grade::S => "S",
            Grade::A => "A",
            Grade::B => "B",
            Grade::C => "C",
            Grade::D => "D",
            Grade::E => "E",
            Grade::F => "F",
            Grade::G => "G",
        }
    }

    /**
     * Returns the next higher grade (S stays at S).
     */
    pub fn upgrade(&self) -> Self {
        match self {
            Grade::S => Grade::S,
            Grade::A => Grade::S,
            Grade::B => Grade::A,
            Grade::C => Grade::B,
            Grade::D => Grade::C,
            Grade::E => Grade::D,
            Grade::F => Grade::E,
            Grade::G => Grade::F,
        }
    }

    /**
     * Numeric index of the grade (G=0 … S=7), used internally for spread and penalty calculations.
     */
    pub fn index(&self) -> u16 {
        match self {
            Grade::G => 0,
            Grade::F => 1,
            Grade::E => 2,
            Grade::D => 3,
            Grade::C => 4,
            Grade::B => 5,
            Grade::A => 6,
            Grade::S => 7,
        }
    }

    /**
     * Constructs a Grade from a numeric index. Values >= 7 map to S.
     */
    fn from_index(idx: u16) -> Self {
        match idx {
            0 => Grade::G,
            1 => Grade::F,
            2 => Grade::E,
            3 => Grade::D,
            4 => Grade::C,
            5 => Grade::B,
            6 => Grade::A,
            _ => Grade::S,
        }
    }

    /**
     * Generate grades for n_slots positions around a best position.
     * The best is always A, and each step away allows 1 more grade of drop.
     */
    fn spread(seed: u64, n_slots: usize, best_pos: usize) -> Vec<Self> {
        const BEST: u16 = 6; // A
        (0..n_slots).map(|pos| {
            let dist = (pos as i32 - best_pos as i32).unsigned_abs() as u16;
            let min_idx = BEST.saturating_sub(dist);
            let range = BEST - min_idx;
            let salt = (pos as u64 + 1) * 10;
            Grade::from_index(min_idx + random(seed + salt, range))
        }).collect()
    }
}

/// Track surface aptitude (Turf / Dirt).
/// One surface is always at least A; the other may receive an additional random penalty.
pub struct Track {
    pub turf: Grade,
    pub dirt: Grade,
}
impl Track {
    /**
     * Generates track aptitudes. The best surface is always at least A.
     * The weaker surface gets an additional random penalty of 0-5 extra grade steps down.
     */
    fn new(seed: u64) -> Self {
        let best_pos = random(seed, 1) as usize;
        let grades = Grade::spread(seed, 2, best_pos);

        let penalty = random(seed.wrapping_add(101), 5) as u16;
        let (turf, dirt) = if grades[0].index() >= grades[1].index() {
            (grades[0], Grade::from_index(grades[1].index().saturating_sub(penalty)))
        } else {
            (Grade::from_index(grades[0].index().saturating_sub(penalty)), grades[1])
        };

        Self { turf, dirt }
    }

    /// Power multiplier for a given track surface grade.
    pub fn multiplier(grade: &Grade) -> f32 {
        match grade {
            Grade::S => 1.02,
            Grade::A => 1.00,
            Grade::B => 0.90,
            Grade::C => 0.80,
            Grade::D => 0.70,
            Grade::E => 0.40,
            Grade::F => 0.20,
            Grade::G => 0.10,
        }
    }
}

/// Distance aptitude across Sprint / Mile / Medium / Long.
/// One distance is always at least A; others drop off based on distance from the best.
pub struct Distance {
    pub sprint: Grade,
    pub mile: Grade,
    pub medium: Grade,
    pub long: Grade,
}
impl Distance {
    /**
     * Generate distances where one is A (the best), and each position
     * further away can drop at most 1 extra grade per step of distance.
     * Best position is chosen randomly from the seed.
     */
    fn new(seed: u64) -> Self {
        let best_pos = random(seed, 3) as usize;
        let grades = Grade::spread(seed, 4, best_pos);
        
        Self {
            sprint: grades[0],
            mile: grades[1],
            medium: grades[2],
            long: grades[3]
        }
    }

    /// Speed multiplier for a given distance grade.
    /// S grade intentionally allows effective speed above the 1200 stat cap.
    pub fn multiplier(grade: &Grade) -> f32 {
        match grade {
            Grade::S => 1.05,
            Grade::A => 1.00,
            Grade::B => 0.90,
            Grade::C => 0.80,
            Grade::D => 0.60,
            Grade::E => 0.40,
            Grade::F => 0.20,
            Grade::G => 0.10,
        }
    }
}

/// Racing strategy aptitude across Front / Pace / Late / End.
/// One style is always at least A; others drop off based on distance from the best.
pub struct Style {
    pub front: Grade,
    pub pace:  Grade,
    pub late:  Grade,
    pub end:   Grade,
}

/// The racing strategy a player (or bot) chooses before a race.
#[derive(Copy, Clone)]
pub enum StyleChoice {
    Front,
    Pace,
    Late,
    End,
}
impl StyleChoice {
    pub fn to_string(&self) -> &str {
        match self {
            StyleChoice::Front => "Front",
            StyleChoice::Pace  => "Pace",
            StyleChoice::Late  => "Late",
            StyleChoice::End   => "End",
        }
    }

    /// Pick the style with the highest grade index automatically.
    pub fn best_of(style: Style) -> Self {
        let options = [
            (StyleChoice::Front, style.front.index()),
            (StyleChoice::Pace,  style.pace.index()),
            (StyleChoice::Late,  style.late.index()),
            (StyleChoice::End,   style.end.index()),
        ];
        options.iter().max_by_key(|(_, i)| *i).unwrap().0
    }
}
impl Style {
    /**
     * Generates style aptitudes. The best style slot is always at least A.
     * Best position is chosen randomly from the seed.
     */
    fn new(seed: u64) -> Self {
        let best_pos = random(seed, 3) as usize;
        let grades = Grade::spread(seed, 4, best_pos);
        
        Self { 
            front: grades[0],
            pace: grades[1],
            late: grades[2],
            end: grades[3]
        }
    }

    /// Wit multiplier for a given racing style grade.
    pub fn multiplier(grade: &Grade) -> f32 {
        match grade {
            Grade::S => 1.05,
            Grade::A => 1.00,
            Grade::B => 0.80,
            Grade::C => 0.60,
            Grade::D => 0.40,
            Grade::E => 0.20,
            Grade::F => 0.10,
            Grade::G => 0.05,
        }
    }
}
