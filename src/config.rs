use rand::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(default)]
pub struct Config {
    pub theme: String,
    pub seed: u64,
    pub grid_width: u32,
    pub grid_height: u32,
    pub grid_scale: u32,
    pub corner_walls: bool,
    pub tick_length: f64,
    pub food_ticks: u32,
    pub snake_spawn_segments: u32,
    pub snake_segment_despawn_interval: f64,
    pub snake_respawn_delay: f64,
    pub eat_audio: String,
    pub destroy_audio: String,
    pub spawn_food_audio: String,
    pub spawn_snake_audio: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "dracula".into(),
            seed: random(),
            grid_width: 17,
            grid_height: 13,
            grid_scale: 36,
            corner_walls: true,
            tick_length: 0.2,
            food_ticks: 16,
            snake_spawn_segments: 2,
            snake_segment_despawn_interval: 0.1,
            snake_respawn_delay: 0.5,
            eat_audio: "eat.mp3".into(),
            destroy_audio: "destroy.mp3".into(),
            spawn_food_audio: "spawn_food.mp3".into(),
            spawn_snake_audio: "spawn_snake.mp3".into(),
        }
    }
}

#[derive(Deserialize)]
pub struct Theme {
    pub walls: String,
    pub background: String,
    pub snake: String,
    pub food: Vec<String>,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            walls: default_theme_color(),
            background: default_theme_color(),
            snake: default_theme_color(),
            food: vec![default_theme_color()]
        }
    }
}

fn default_theme_color() -> String {
    String::from("ff00ff")
}