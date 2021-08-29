use std::{collections::HashMap, convert::TryInto};

use rand::prelude::*;
use serde::{de::Visitor, Deserialize, Deserializer};

use crate::Direction;

#[derive(Deserialize)]
#[serde(default)]
pub struct Config {
    pub theme: String,
    pub seed: u64,
    pub map: Map,
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
            map: Default::default(),
            grid_scale: 36,
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
#[serde(tag = "type")]
pub enum Map {
    #[serde(rename = "box")]
    Box {
        width: u32,
        height: u32,
        corner_walls: bool,
    },
    #[serde(rename = "custom")]
    Custom {
        #[serde(deserialize_with = "deserialize_map_data")]
        data: MapData,
    },
}

impl Default for Map {
    fn default() -> Self {
        Self::Box {
            width: 17,
            height: 13,
            corner_walls: true,
        }
    }
}

pub struct MapData {
    pub width: u32,
    pub height: u32,
    cells: HashMap<(u32, u32), Cell>,
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
        Self {
            walls: default_theme_color(),
            background: default_theme_color(),
            snake: default_theme_color(),
            food: vec![default_theme_color()],
        }
    }
}

fn default_theme_color() -> String {
    String::from("ff00ff")
}

fn deserialize_map_data<'de, D>(deserializer: D) -> Result<MapData, D::Error>
where
    D: Deserializer<'de>,
{
    struct MapDataVisitor;

    impl<'de> Visitor<'de> for MapDataVisitor {
        type Value = MapData;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a multi-line string composed of ' ' and '#'")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            fn to_u32_in_range<E: serde::de::Error>(value: usize, name: &str) -> Result<u32, E> {
                value
                    .try_into()
                    .map_err(|_| E::custom(format!("{} dimension is too big", name)))
            }

            let mut cells = HashMap::new();
            let mut width = 0u32;
            let mut height = 0u32;

            for (row, line) in value.lines().enumerate() {
                let row = to_u32_in_range(row, "Vertical")?;

                for (column, char) in line.chars().enumerate() {
                    let column = to_u32_in_range(column, "Horizontal")?;

                    cells.insert(
                        (column as u32, row as u32),
                        match char {
                            '#' => Cell::Wall,
                            '^' => Cell::Spawn(Direction::Up),
                            'v' => Cell::Spawn(Direction::Down),
                            '<' => Cell::Spawn(Direction::Left),
                            '>' => Cell::Spawn(Direction::Right),
                            ' ' => Cell::Empty,
                            other => {
                                return Err(E::custom(format!("Unknown cell type {:?}", other)))
                            }
                        },
                    );

                    width = width.max(column + 1);
                    height = height.max(row + 1);
                }
            }

            Ok(MapData {
                width,
                height,
                cells,
            })
        }
    }

    deserializer.deserialize_str(MapDataVisitor)
}
