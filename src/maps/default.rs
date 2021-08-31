use crate::{
    config::{Cell, Map, MapData},
    Direction,
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct DefaultMap {
    pub width: u32,
    pub height: u32,
    corner_walls: u32,
    corner_walls_offset: i32,
}

impl Default for DefaultMap {
    fn default() -> Self {
        Self {
            width: 17,
            height: 13,
            corner_walls: 2,
            corner_walls_offset: 2,
        }
    }
}

#[typetag::serde]
impl Map for DefaultMap {
    fn get_map_data(&self, _generator: &mut rand_pcg::Pcg64) -> MapData {
        let width = self.width;
        let height = self.height;
        let corner_walls = self.corner_walls;
        let corner_walls_offset = self.corner_walls_offset;
        MapData {
            width,
            height,
            cells: {
                let mut cells = HashMap::new();
                let corner_walls_offset = corner_walls_offset as u32;
                for x in 0..width {
                    for y in 0..height {
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
        }
    }
    fn get_dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}
