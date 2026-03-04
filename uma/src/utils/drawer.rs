use std::io::{self, Write};
use crate::classes::uma::{Uma, StatType, performance};

pub fn clear_console() {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();
}
use crate::classes::race::Race;

/// Pads or truncates a string to exactly `width` characters.
pub fn pad(text: &str, width: usize) -> String {
    if text.len() >= width {
        text[..width].to_string()
    } else {
        format!("{:<width$}", text, width = width)
    }
}

/**
 * Draw a progress bar
 * - max: the maximum value
 * - current: the current value
 * - length: the number of characters inside the bar
 */
pub fn draw_competitors(race: &Race) {
    let bar = "-".repeat(8);
    println!();
    println!("{} Competitors {}", bar, bar);
    for (i, runner) in race.get_runners().iter().enumerate() {
        let tag = if runner.is_human() { "(YOU)" } else { "     " };
        println!(
            "  {}. {} {} | Rank: {} | Mood: {} | Style: {} ({})",
            i + 1,
            tag,
            pad(runner.get_name(), 20),
            pad(performance(runner.get_points()), 3),
            pad(runner.get_mood().to_string(), 12),
            pad(runner.get_chosen_style().to_string(), 5),
            runner.get_chosen_style_grade().to_string(),
        );
    }
    println!();
}

/**
 * Draw a progress bar
 * - max: the maximum value
 * - current: the current value
 * - length: the number of characters inside the bar
 */
pub fn draw_bar(max: u8, current: u8, length: u8) -> String {
    let filled = ((current as f32 / max as f32) * length as f32).round() as u8;
    let empty = length.saturating_sub(filled);

    let bar: String = "█".repeat(filled as usize) + &"░".repeat(empty as usize);
    format!("[{}]", bar)
}

/**
 * Draw the game view, showing the current turn, energy, mood, and stats with failure chances.
 * - uma: the Uma object containing all the relevant data
 * This function is called at the start of each turn to show the current state of the Uma.
 * It displays the Uma's name, current turn, energy bar, mood, and stats with their respective failure chances.
 */
pub fn draw_game_view(uma: &Uma) {
    let bar = "-".repeat(8);

    if uma.get_turns_to_race() > 0 {
        println!();
        println!("{} Turns before the race {} {}", bar, uma.get_turns_to_race(), bar);
        println!("Training {}...", uma.get_name());
        println!();
        println!("Energy {}", draw_bar(uma.get_max_energy(), uma.get_energy(), 32));
        println!("Mood: {}", uma.get_mood().to_string());
        println!();
        draw_stats(uma,true);
    } else {
        println!();
        println!("{} Prepare for the race {}", bar, bar);
        println!();
        println!("Mood: {}", uma.get_mood().to_string());
        println!();
        draw_stats(uma,false);
    }
}

/**
 * Draw the game options menu, showing the available actions for the player.
 * This function is called after the game view to prompt the player for their next action. It lists all the possible actions, including training each stat, resting, showing full stats, and exiting the game. The options are clearly labeled with their corresponding commands for easy input.
 * The menu is designed to be user-friendly and provides a clear overview of the player's choices for the next turn.
 */
pub fn draw_game_options(uma: &Uma) {
    let bar = "=".repeat(4);
    let is_race_turn = uma.get_turns_to_race() <= 0;

    println!();
    println!("{} Actions {}", bar, bar);

    if !is_race_turn {
        println!("  spe. Train Speed");
        println!("  sta. Train Stamina");
        println!("  pow. Train Power");
        println!("  gut. Train Guts");
        println!("  wit. Train Wit");
        println!();
        println!("  res. Rest");
        println!("  rec. Recreation");
    } else {
        println!("  race. Start Race");
    }

    println!();
    println!("  show. Show Stats");
    println!("  exit. Exit");
    println!();
}

/**
 * Draw the Uma's stats with optional failure chances.
 * - uma: the Uma object containing all the relevant data
 * - add_failure_chance: whether to include failure chances in the output
 * This function is used to display the Uma's stats in a clear format. If add_failure
 * chance is true, it also shows the failure chance for each stat, which is calculated based on the current stat value.
 * The output includes the stat name, current value, and failure chance percentage for each of the five stats: Speed, Stamina, Power, Guts, and Wit.
 */
pub fn draw_stats(uma: &Uma, add_failure_chance: bool) {
    if add_failure_chance {
        println!("  Speed:   {} | Fail: {}%", uma.stats.speed.to_string(),   uma.failure_chance(&StatType::Speed));
        println!("  Stamina: {} | Fail: {}%", uma.stats.stamina.to_string(), uma.failure_chance(&StatType::Stamina));
        println!("  Power:   {} | Fail: {}%", uma.stats.power.to_string(),   uma.failure_chance(&StatType::Power));
        println!("  Guts:    {} | Fail: {}%", uma.stats.guts.to_string(),    uma.failure_chance(&StatType::Guts));
        println!("  Wit:     {} | Fail: {}%", uma.stats.wit.to_string(),     uma.failure_chance(&StatType::Wit));
    } else {
        println!("  Speed:   {}", uma.stats.speed.to_string());
        println!("  Stamina: {}", uma.stats.stamina.to_string());
        println!("  Power:   {}", uma.stats.power.to_string());
        println!("  Guts:    {}", uma.stats.guts.to_string());
        println!("  Wit:     {}", uma.stats.wit.to_string());
    }
}

/**
 * Draw the full stats view, showing all stats and track/distance/style preferences.
 * - uma: the Uma object containing all the relevant data
 * This function is called at the end of the training period to show the final stats of the Uma. It displays the Uma's name, all five stats (Speed, Stamina, Power, Guts, Wit) without failure chances, and the track, distance, and style preferences. The track preferences include Turf and Dirt grades, the distance preferences include Sprint, Mile, Medium, and Long grades, and the style preferences include Front, Pace, Late, and End grades.
 * The output is formatted to clearly show the Uma's overall performance and specialization in different areas.
 */
pub fn draw_full_stats(uma: &Uma) {
    println!("{}'s Stats:", uma.get_name());
    draw_stats(uma, false);
    println!();
    println!("{} Rank", performance(uma.get_points()));
    println!();
    println!("  Turns: {} | Races: {} | Wins: {}", uma.get_turns(), uma.get_races(), uma.get_wins());
    println!();
    println!("  Track: Turf: {} | Dirt: {}", uma.track.turf.to_string(), uma.track.dirt.to_string());
    println!(
        "  Distance: Sprint: {} | Mile: {} | Medium: {} | Long: {}",
        uma.distance.sprint.to_string(),
        uma.distance.mile.to_string(),
        uma.distance.medium.to_string(),
        uma.distance.long.to_string()
    );
    println!(
        "  Style: Front: {} | Pace: {} | Late: {} | End: {}",
        uma.style.front.to_string(),
        uma.style.pace.to_string(),
        uma.style.late.to_string(),
        uma.style.end.to_string()
    );
}
