use nid::{alphabet::Base62Alphabet, Nanoid};
use rand::Rng;
use regex::Regex;

pub fn generate_uuid() -> String {
    let nid: Nanoid<8, Base62Alphabet> = Nanoid::new();
    nid.to_string()
}

pub fn generate_valid_username() -> String {
    let username = generate_username();
    if is_valid_room_or_username(&username) {
        username
    } else {
        generate_valid_username()
    }
}

#[allow(dead_code)]
pub fn generate_valid_room_name() -> String {
    let room_name = generate_room_name();
    if is_valid_room_or_username(&room_name) {
        room_name
    } else {
        generate_valid_room_name()
    }
}

pub fn is_valid_room_or_username(name: &str) -> bool {
    let re = Regex::new(r"^[a-zA-Z0-9\-]{3,20}$").unwrap();
    re.is_match(&name)
}

fn generate_room_name() -> String {
    format!(
        "{}-{}-{}",
        random_adjective(),
        random_adjective(),
        random_noun(),
    )
}

fn generate_username() -> String {
    format!("{}-{}-{}", random_adjective(), random_noun(), random_num())
}

fn random_num() -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(10..99)
}

fn random_adjective() -> String {
    get_random_item(vec![
        "barren", "chilly", "cold", "distant", "eerie", "frozen", "haunted", "hidden", "hollow",
        "lonely", "misty", "moody", "mystic", "quiet", "secret", "silent", "shrouded", "stark",
        "subtle", "sullen", "veiled", "velvet", "windy",
    ])
}

fn random_noun() -> String {
    get_random_item(vec![
        "ash", "beam", "blossom", "castle", "cliff", "cloud", "crow", "crypt", "dust", "field",
        "flame", "fog", "frost", "ghost", "gloom", "glow", "grave", "leaf", "marsh", "mist",
        "moon", "night", "path", "raven", "root", "ruin", "sage", "shade", "snow", "star", "stone",
        "storm", "stream", "thorn", "wolf",
    ])
}

fn get_random_item(list: Vec<&str>) -> String {
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..list.len());
    list[index].to_string()
}
