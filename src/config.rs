use std::{collections::HashMap, convert::TryInto};

use rand::prelude::*;
use rand_pcg::Pcg64;
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
        corner_walls: u32,
        corner_walls_offset: i32,
    },
    #[serde(rename = "corridors")]
    Corridors {
        width: u32,
        height: u32,
        corridor_width: u32,
        corridor_height: u32,
        top_corridor_offset: i32,
        bottom_corridor_offset: i32,
        wall_variance: f32,
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
            corner_walls: 2,
            corner_walls_offset: 2,
        }
    }
}

impl Map {
    pub fn get_map_data(&self, generator: &mut Pcg64) -> MapData {
        match self {
            Self::Box {
                width,
                height,
                corner_walls,
                corner_walls_offset,
            } => MapData {
                width: *width,
                height: *height,
                cells: {
                    let mut cells = HashMap::new();
                    let corner_walls_offset = *corner_walls_offset as u32;
                    for x in 0..*width {
                        for y in 0..*height {
                            cells.insert((x, y), {
                                if x == 0
                                    || x == width - 1
                                    || y == 0
                                    || y == height - 1
                                    // Bottom-left corner wall
                                    || (x >= corner_walls_offset
                                        && x < corner_walls_offset + corner_walls
                                        && y >= height - corner_walls_offset - corner_walls
                                        && y < height - corner_walls_offset)
                                    // Top-left corner wall
                                    || (x >= corner_walls_offset
                                        && x < corner_walls_offset + corner_walls
                                        && y >= corner_walls_offset
                                        && y < corner_walls_offset + corner_walls)
                                    // Bottom-right corner wall
                                    || (x >= width - corner_walls_offset - corner_walls
                                        && x < width - corner_walls_offset
                                        && y >= height - corner_walls_offset - corner_walls
                                        && y < height - corner_walls_offset)
                                    // Top-right corner wall
                                    || (x >= width - corner_walls_offset - corner_walls
                                        && x < width - corner_walls_offset
                                        && y >= corner_walls_offset
                                        && y < corner_walls_offset + corner_walls)
                                {
                                    Cell::Wall
                                } else if x == width / 2 - 1 && y == height / 2 {
                                    Cell::Spawn(Direction::Left)
                                } else if x == width / 2 + 1 && y == height / 2 {
                                    Cell::Spawn(Direction::Right)
                                } else {
                                    Cell::Empty
                                }
                            });
                        }
                    }
                    cells
                },
            },
            Self::Corridors {
                width,
                height,
                corridor_width,
                corridor_height,
                top_corridor_offset,
                bottom_corridor_offset,
                wall_variance,
            } => MapData {
                width: *width,
                height: *height,
                cells: {
                    let mut cells = HashMap::new();
                    let corridor_width = *corridor_width as u32;
                    let mut top_wall_heights = HashMap::<u32, u32>::new();
                    let mut bottom_wall_heights = HashMap::<u32, u32>::new();
                    let get_wall_height =
                        |hash_map: &mut HashMap<u32, u32>, generator: &mut Pcg64, x: u32| {
                            *hash_map.entry(x).or_insert_with(|| {
                                let corridor_height = *corridor_height as f32;
                                (corridor_height * (1.0 - wall_variance)
                                    + corridor_height * wall_variance * generator.gen::<f32>())
                                    as u32
                            })
                        };
                    for x in 0..*width {
                        let mut blocked = true;
                        for y in 0..*height {
                            cells.insert((x, y), {
                                if x == width / 2 - 1 && y == height / 2 {
                                    blocked = false;
                                    Cell::Spawn(Direction::Left)
                                } else if x == width / 2 + 1 && y == height / 2 {
                                    blocked = false;
                                    Cell::Spawn(Direction::Right)
                                } else if x == 0
                                    || x == width - 1
                                    || y == 0
                                    || y == height - 1
                                    || ((x as i32 - top_corridor_offset)
                                        % (corridor_width as i32 + 1)
                                        == 0
                                        && x > 2
                                        && x < width - corridor_width - 1
                                        && y < get_wall_height(&mut top_wall_heights, generator, x)
                                            + 1)
                                    || ((x as i32 - bottom_corridor_offset)
                                        % (corridor_width as i32 + 1)
                                        == 0
                                        && x > 2
                                        && x < width - corridor_width - 1
                                        && y > height
                                            - get_wall_height(
                                                &mut bottom_wall_heights,
                                                generator,
                                                x,
                                            )
                                            - 2)
                                {
                                    Cell::Wall
                                } else {
                                    blocked = false;
                                    Cell::Empty
                                }
                            });
                        }
                        // Check for blocked columns
                        if blocked && x > 0 && x < width - 1 {
                            let gap = generator.gen_range(1..(height - 1));
                            cells.insert((x, gap), Cell::Empty);
                        }
                    }
                    cells
                },
            },
            Self::Custom { data } => data.clone(),
        }
    }
}

#[derive(Clone)]
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
        const DEFAULT_COLOR: &str = "ff00ff";
        Self {
            walls: DEFAULT_COLOR.into(),
            background: DEFAULT_COLOR.into(),
            snake: DEFAULT_COLOR.into(),
            food: vec![DEFAULT_COLOR.into()],
        }
    }
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
