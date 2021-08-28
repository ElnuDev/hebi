use rand::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default = "default_seed")]
    pub seed: u64,

    #[serde(default = "default_grid_width")]
    pub grid_width: u32,

    #[serde(default = "default_grid_height")]
    pub grid_height: u32,

    #[serde(default = "default_grid_scale")]
    pub grid_scale: u32,

    #[serde(default = "default_corner_walls")]
    pub corner_walls: bool,

    #[serde(default = "default_tick_length")]
    pub tick_length: f64,

    #[serde(default = "default_food_ticks")]
    pub food_ticks: u32,

    #[serde(default = "default_snake_spawn_segments")]
    pub snake_spawn_segments: u32,

    #[serde(default = "default_snake_segment_despawn_interval")]
    pub snake_segment_despawn_interval: f64,

    #[serde(default = "default_snake_respawn_delay")]
    pub snake_respawn_delay: f64,

    #[serde(default = "default_eat_audio")]
    pub eat_audio: String,

    #[serde(default = "default_destroy_audio")]
    pub destroy_audio: String,

    #[serde(default = "default_spawn_food_audio")]
    pub spawn_food_audio: String,

    #[serde(default = "default_spawn_snake_audio")]
    pub spawn_snake_audio: String,
}

fn default_theme() -> String {
    String::from("dracula")
}

fn default_seed() -> u64 {
    random()
}

fn default_grid_width() -> u32 {
    17
}

fn default_grid_height() -> u32 {
    13
}

fn default_grid_scale() -> u32 {
    36
}

fn default_corner_walls() -> bool {
    true
}

fn default_tick_length() -> f64 {
    0.2
}

fn default_food_ticks() -> u32 {
    16
}

fn default_snake_spawn_segments() -> u32 {
    2
}

fn default_snake_segment_despawn_interval() -> f64 {
    0.1
}

fn default_snake_respawn_delay() -> f64 {
    0.5
}

fn default_eat_audio() -> String {
    String::from("eat.mp3")
}

fn default_destroy_audio() -> String {
    String::from("destroy.mp3")
}

fn default_spawn_food_audio() -> String {
    String::from("spawn_food.mp3")
}

fn default_spawn_snake_audio() -> String {
    String::from("spawn_snake.mp3")
}

#[derive(Deserialize)]
pub struct Theme {
    #[serde(default = "default_theme_color")]
    pub walls: String,

    #[serde(default = "default_theme_color")]
    pub background: String,

    #[serde(default = "default_theme_color")]
    pub snake: String,

    #[serde(default = "default_theme_color_vec")]
    pub food: Vec<String>,
}

fn default_theme_color() -> String {
    String::from("ff00ff")
}

fn default_theme_color_vec() -> Vec<String> {
    vec![default_theme_color()]
}
