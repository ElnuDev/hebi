use crate::{maps::*, Direction};

use bevy::prelude::KeyCode;
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
                Binding::Keyboard { key: KeyCode::Up },
                Binding::Keyboard { key: KeyCode::W },
                Binding::Keyboard { key: KeyCode::K },
                Binding::Keyboard { key: KeyCode::Numpad8 }, // numpad up with num lock
            ],
            down: vec![
                Binding::Keyboard { key: KeyCode::Down },
                Binding::Keyboard { key: KeyCode::S },
                Binding::Keyboard { key: KeyCode::J },
                Binding::Keyboard { key: KeyCode::Numpad2 }, // numpad down with num lock
            ],
            left: vec![
                Binding::Keyboard { key: KeyCode::Left },
                Binding::Keyboard { key: KeyCode::A },
                Binding::Keyboard { key: KeyCode::H },
            ],
            right: vec![
                Binding::Keyboard { key: KeyCode::Right },
                Binding::Keyboard { key: KeyCode::D },
                Binding::Keyboard { key: KeyCode::L },
                Binding::Keyboard { key: KeyCode::Numpad6 }, // numpad right with num lock
            ],
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "device")]
pub enum Binding {
    #[serde(rename = "keyboard")]
    Keyboard { key: KeyCode }
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
