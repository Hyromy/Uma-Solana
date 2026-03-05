use solana_program::hash::hash;

pub fn random(seed: u64, limit: u16) -> u16 {
    let seed_bytes = seed.to_le_bytes();
    let hashed     = hash(&seed_bytes);
    let value      = u16::from_le_bytes([hashed.to_bytes()[0], hashed.to_bytes()[1]]);
    value % (limit + 1)
}

pub fn get_seed() -> u64 {
    use solana_program::clock::Clock;
    use solana_program::sysvar::Sysvar;
    Clock::get().unwrap().slot
}

pub fn salt(string: String) -> u64 {
    string
        .bytes()
        .enumerate()
        .map(|(i, b)| (b as u64) << (i % 8))
        .sum()
}

pub fn random_uma_name(seed_salt: String) -> String {
    let names = [
        "Special Week",     "Silence Suzuka", "Tokai Teio",     "Mejiro McQueen",
        "Gold Ship",        "Vodka",          "Daiwa Scarlet",  "Grass Wonder",
        "El Condor Pasa",   "Rice Shower",    "Mihono Bourbon", "Kitasan Black",
        "Satono Diamond",   "Nice Nature",    "Twin Turbo",     "Oguri Cap",
        "Tamamo Cross",     "Symboli Rudolf", "Air Groove",     "Agnes Tachyon",
        "Manhattan Cafe",   "Mayano Top Gun", "Narita Brian",   "Winning Ticket",
        "Sakura Bakushin O","Haru Urara",     "King Halo",      "Maruzensky",
        "Taiki Shuttle",    "Mejiro Dober",   "Narita Top Road","Jungle Pocket",
    ];
    let index = random(get_seed() + salt(seed_salt), (names.len() - 1) as u16) as usize;
    names[index].to_string()
}

pub fn random_track_name() -> String {
    let tracks = [
        "Tokyo",     "Nakayama",    "Kyoto",       "Hanshin",
        "Chukyo",    "Sapporo",     "Hakodate",    "Niigata",
        "Fukushima", "Kokura",      "Ohi",         "Kawasaki",
        "Funabashi", "Urawa",       "Mombetsu",    "Morioka",
        "Sonoda",    "Kochi",       "Saga",        "Nagoya",
        "Kasamatsu", "Kanazawa",    "Longchamp",   "Dubai Meydan",
        "Sha Tin",   "Santa Anita", "Chiba",       "Hokkaido",
        "Hyogo",     "Miyagi",      "Gifu",        "Saitama",
    ];
    let index = random(get_seed(), (tracks.len() - 1) as u16) as usize;
    tracks[index].to_string()
}

pub fn random_distance() -> u16 {
    let distances = [
        1000u16, 1200, 1400,
        1600, 1800,
        2000, 2200, 2400,
        2500, 3000, 3200, 3600,
    ];
    let index = random(get_seed(), (distances.len() - 1) as u16) as usize;
    distances[index]
}
