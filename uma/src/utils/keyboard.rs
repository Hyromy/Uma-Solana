use std::io::{self, Write};
use crate::classes::uma::{Uma, StatType, StyleChoice};
use crate::utils::drawer::{draw_full_stats, draw_competitors, clear_console};
use crate::classes::race::{prepare_to_race, Race};

pub fn input() -> String {
    print!("> ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn train_output(uma: &mut Uma, stat: StatType) {
    let fc = uma.failure_chance(&stat);
    let points = uma.train(stat, fc);
    if points > 0 {
        println!("Trained {} for {} points!", stat.to_string(), points);
    } else {
        println!("{} doesnt train well.", uma.get_name());
    }
}

fn race_submenu(mut race: Race) -> Uma {
    loop {
        let bar = "=".repeat(8);
        let player = race.get_runners().iter().find(|r| r.is_human()).unwrap();
        let sc = player.get_chosen_style();
        let grade = player.get_chosen_style_grade();
        let track = race.get_track();
        let dist_grade = player.get_distance_grade(&track.distance_type);
        let surf_grade = player.get_surface_grade(&track.surface);
        println!();
        println!("{} {} {}", bar, track.to_string(), bar);
        println!("  Style:    {} ({})", sc.to_string(), grade.to_string());
        println!("  Distance: {}", dist_grade.to_string());
        println!("  Surface:  {}", surf_grade.to_string());
        println!();
        println!("  run. Start the race");
        println!();
        println!("  com. See competitors");
        println!("  sty. Change racing style");
        println!("  show. Show stats");
        println!();
        let option = input();
        clear_console();
        match option.as_str() {
            "com" => { draw_competitors(&race); }
            "sty" => {
                let player = race.get_runners().iter().find(|r| r.is_human()).unwrap();
                println!("Choose style:");
                println!("  fro. Front  ({})", player.style.front.to_string());
                println!("  pac. Pace   ({})", player.style.pace.to_string());
                println!("  lat. Late   ({})", player.style.late.to_string());
                println!("  end. End    ({})", player.style.end.to_string());
                let choice = match input().as_str() {
                    "fro" => Some(StyleChoice::Front),
                    "pac" => Some(StyleChoice::Pace),
                    "lat" => Some(StyleChoice::Late),
                    "end" => Some(StyleChoice::End),
                    _ => { println!("Invalid style."); None }
                };
                if let Some(sc) = choice {
                    let player = race.get_runners_mut().iter_mut().find(|r| r.is_human()).unwrap();
                    player.set_style(sc);
                    println!("Style set to {}!", sc.to_string());
                }
            }
            "run" => {
                let name = race.get_runners().iter().find(|r| r.is_human()).unwrap().get_name().to_string();
                println!("The race has started! Good luck {}!", name);
                let position = race.run();
                let mut all = race.into_runners();
                let player_idx = all.iter().position(|r| r.is_human()).unwrap();
                let mut player = all.remove(player_idx);
                player.race_result(position);
                return player;
            }
            "show" => {
                let player = race.get_runners().iter().find(|r| r.is_human()).unwrap();
                draw_full_stats(player);
            }
            _ => println!("Type 'com' or 'run'."),
        }
    }
}

pub fn make_choice(mut uma: Uma, option: &str) -> Uma {
    match option {
        "spe" => { train_output(&mut uma, StatType::Speed); }
        "sta" => { train_output(&mut uma, StatType::Stamina); }
        "pow" => { train_output(&mut uma, StatType::Power); }
        "gut" => { train_output(&mut uma, StatType::Guts); }
        "wit" => { train_output(&mut uma, StatType::Wit); }

        "res" => { uma.rest(); }
        "rec" => { uma.recreation(); }

        "race" => {
            match prepare_to_race(uma) {
                Err(returned) => { uma = returned; }
                Ok(race) => { uma = race_submenu(race); }
            }
        }

        "show" => { draw_full_stats(&uma); }
        "exit" => {
            println!("See you later!");
            std::process::exit(0);
        }
        _ => println!("Invalid choice, please try again."),
    }
    uma
}
