use crate::{maps::*, Direction};

use rand::prelude::*;
use rand_pcg::Pcg64;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;

#[derive(Deserialize)]
#[serde(default)]
pub struct Config {
    pub theme: String,
    pub seed: u64,
    pub map: Box<dyn Map>,
    pub controls: Controls,
    pub grid_scale: u32,
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
            map: Box::new(DefaultMap::default()),
            controls: Default::default(),
            grid_scale: 36,
            tick_length: 0.2,
            food_ticks: 16,
            snake_spawn_segments: 2,
            snake_segment_despawn_interval: 0.1,
            snake_respawn_delay: 0.5,
            eat_audio: "eat.wav".into(),
            destroy_audio: "destroy.wav".into(),
            spawn_food_audio: "spawn_food.wav".into(),
            spawn_snake_audio: "spawn_snake.wav".into(),
        }
    }
}

#[derive(Deserialize)]
#[serde(default)]
pub struct Controls {
    pub up: Vec<Binding>,
    pub down: Vec<Binding>,
    pub left: Vec<Binding>,
    pub right: Vec<Binding>,
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            up: vec![
                Binding::Keyboard { code: 57416 }, // up arrow key
                Binding::Keyboard { code: 17 }, // W
                Binding::Keyboard { code: 37 }, // K
                Binding::Keyboard { code: 72 }, // numpad up
            ],
            down: vec![
                Binding::Keyboard { code: 57424 }, // down arrow key
                Binding::Keyboard { code: 31 }, // S
                Binding::Keyboard { code: 36 }, // J
                Binding::Keyboard { code: 80 }, // numpad down
            ],
            left: vec![
                Binding::Keyboard { code: 57419 }, // left arrow key
                Binding::Keyboard { code: 30 }, // A
                Binding::Keyboard { code: 35 }, // H
                Binding::Keyboard { code: 75 }, // numpad left
            ],
            right: vec![
                Binding::Keyboard { code: 57421 }, // right arrow key
                Binding::Keyboard { code: 32 }, // D
                Binding::Keyboard { code: 38 }, // L
                Binding::Keyboard { code: 77 }, // numpad right
            ],
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "device")]
pub enum Binding {
    #[serde(rename = "keyboard")]
    Keyboard { code: u32 }
}

#[derive(Clone, Serialize)]
pub struct MapData {
    pub width: u32,
    pub height: u32,
    #[serde(serialize_with = "serialize_cells")]
    pub cells: HashMap<(u32, u32), Cell>,
}

fn serialize_cells<S>(_cells: &HashMap<(u32, u32), Cell>, _s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    unimplemented!("Map data cells serialization is unimplemented!");
}

impl MapData {
    pub fn iter(&self) -> impl Iterator<Item = (u32, u32, Cell)> + '_ {
        self.cells.iter().map(|((x, y), cell)| (*x, *y, *cell))
    }
}

#[derive(Clone, Copy)]
pub enum Cell {
    Empty,
    Wall,
    Spawn(Direction),
}

#[derive(Deserialize)]
#[serde(default)]
pub struct Theme {
    pub walls: String,
    pub background: String,
    pub snake: String,
    pub food: Vec<String>,
}

impl Default for Theme {
    fn default() -> Self {
        const DEFAULT_COLOR: &str = "ff00ff";
        Self {
            walls: DEFAULT_COLOR.into(),
            background: DEFAULT_COLOR.into(),
            snake: DEFAULT_COLOR.into(),
            food: vec![DEFAULT_COLOR.into()],
        }
    }
}

#[typetag::serde(tag = "type")]
pub trait Map {
    fn get_map_data(&self, generator: &mut Pcg64) -> MapData;
    fn get_dimensions(&self) -> (u32, u32);
}
