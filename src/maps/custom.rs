use crate::{
    config::{Cell, MapData, MapType},
    Direction,
};

use rand_pcg::Pcg64;
use serde::{de::Visitor, Deserialize, Deserializer};
use std::{collections::HashMap, convert::TryInto};

#[derive(Deserialize)]
pub struct CustomMap {
    #[serde(deserialize_with = "deserialize_map_data")]
    pub data: MapData,
}

impl MapType for CustomMap {
    fn get_map_data(&self, _generator: &mut Pcg64) -> MapData {
        self.data.clone()
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
