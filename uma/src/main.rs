mod classes {
    pub mod uma;
    pub mod racecourse;
    pub mod race;
    pub mod bot;
}
mod utils {
    pub mod drawer;
    pub mod keyboard;
    pub mod random;
}

use classes::uma::Uma;
use utils::drawer::{
    draw_game_view,
    draw_game_options,
    draw_full_stats,
    clear_console,
};
use utils::keyboard::{
    input,
    make_choice,
};
use utils::random::random_uma_name;

fn main() {
    println!("Welcome to Uma-Solana!");
    println!("This game is a cut version text-based of Umamusume Pretty Derby, where you train your Uma to prepare for races, and you can choose to train different stats or rest each turn. The goal is to win races");
    println!("If you lose, your Uma may be retired and you can start a new one. Good luck!");
    println!("First, please enter the name of your Uma (set empty to get a random name):");
    let mut name = input();

    if name.is_empty() {
        name = random_uma_name(String::from("trainer"));
    }

    let mut uma = Uma::new(name, true);

    loop {
        if uma.get_is_end() {
            clear_console();
            draw_full_stats(&uma);
            println!();
            println!("Your Uma has been retired. Press enter to exit.");
            input();
            std::process::exit(0);
        }

        draw_game_view(&uma);
        draw_game_options(&uma);
        let option = input();
        clear_console();
        uma = make_choice(uma, &option);
        println!();
    }
}
