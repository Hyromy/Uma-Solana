use crate::utils::random::{random_track_name, random_distance, get_seed, random, salt};

pub struct Racecourse {
    pub name: String,
    pub distance: u16,
    pub distance_type: DistanceType,
    pub surface: Surface,
}
impl Racecourse {
    pub fn new() -> Self {
        let distance = random_distance();
        Self {
            name: random_track_name(),
            distance,
            distance_type: DistanceType::from_distance(distance),
            surface: Surface::random(),
        }
    }

    pub fn to_string(&self) -> String {
        format!(
            "{} | {} | {}m ({})",
            self.name,
            self.surface.to_string(),
            self.distance,
            self.distance_type.to_string()
        )
    }
}

pub enum Surface {
    Turf,
    Dirt,
}
impl Surface {
    fn random() -> Self {
        if random(get_seed(), salt(String::from("surface")) as u16) % 2 == 0 {
            Surface::Turf
        } else {
            Surface::Dirt
        }
    }

    pub fn to_string(&self) -> &str {
        match self {
            Surface::Turf => "Turf",
            Surface::Dirt => "Dirt",
        }
    }
}

pub enum DistanceType {
    Sprint,
    Mile,
    Medium,
    Long,
}
impl DistanceType {
    fn from_distance(distance: u16) -> Self {
        match distance {
            1000 | 1200 | 1400        => DistanceType::Sprint,
            1600 | 1800               => DistanceType::Mile,
            2000 | 2200 | 2400        => DistanceType::Medium,
            2500 | 3000 | 3200 | 3600 => DistanceType::Long,
            _ => panic!("Invalid distance"),
        }
    }

    pub fn to_string(&self) -> &str {
        match self {
            DistanceType::Sprint => "Sprint",
            DistanceType::Mile   => "Mile",
            DistanceType::Medium => "Medium",
            DistanceType::Long   => "Long",
        }
    }
}
