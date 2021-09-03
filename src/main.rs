#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod maps;

use crate::config::*;
use bevy::core::FixedTimestep;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::ElementState;
use bevy::prelude::*;
use bevy::utils::HashMap;
use rand::prelude::*;
use rand::seq::SliceRandom;
use rand_pcg::Pcg64;
use serde::de::DeserializeOwned;
use std::fs;

const TITLE: &str = "Hebi";
const MISSING_COLOR: Color = Color::FUCHSIA;

#[derive(SystemLabel, Debug, Hash, PartialEq, Eq, Clone)]
enum Labels {
    Moving,
    Respawning,
}

struct GridDimensions {
    width: u32,
    height: u32,
    scale: u32,
}

struct DirectionalControls {
    scan_codes: HashMap<u32, Direction>,
}

impl DirectionalControls {
    fn from_keyboard(&self, event: &KeyboardInput) -> Option<Direction> {
        self.scan_codes
            .get(&event.scan_code)
            .cloned()
    }
}

impl FromWorld for DirectionalControls {
    fn from_world(world: &mut World) -> Self {
        let config = world
            .get_non_send_resource::<Config>()
            .expect("Missing configuration from which to create control mapping");

        let mut result = Self {
            scan_codes: Default::default(),
        };

        let by_direction = [
            (Direction::Up, &config.controls.up),
            (Direction::Down, &config.controls.down),
            (Direction::Left, &config.controls.left),
            (Direction::Right, &config.controls.right),
        ];

        for (direction, bindings) in by_direction {
            for binding in bindings {
                match binding {
                    Binding::Keyboard { code } => {
                        result.scan_codes.insert(*code, direction);
                    }
                }
            }
        }

        result
    }
}

struct Random {
    snake_spawn_generator: Pcg64,
    food_spawn_generator: Pcg64,
    environment_generator: Pcg64,
}

impl Random {
    fn new(config: &Config) -> Self {
        let generator = || Pcg64::seed_from_u64(config.seed);
        Random {
            snake_spawn_generator: generator(),
            food_spawn_generator: generator(),
            environment_generator: generator(),
        }
    }
}

fn main() {
    fn read_toml_file<T: DeserializeOwned + Default>(path: &str) -> T {
        let result = fs::read_to_string(path)
            .map_err(|error| format!("Failed to load {:?}: {}", path, error))
            .and_then(|contents| {
                toml::from_str(&contents)
                    .map_err(|error| format!("Failed to parse {:?}: {}", path, error))
            });

        match result {
            Ok(value) => value,
            Err(error) => {
                eprintln!("{}", error);
                Default::default()
            }
        }
    }

    let config: Config = read_toml_file("config.toml");
    let theme: Theme = read_toml_file(&format!("themes/{}.toml", config.theme));

    let (grid_width, grid_height) = config.map.get_dimensions();

    let grid_scale = config.grid_scale;

    App::build()
        .add_startup_system(setup.system())
        .add_system(snake_movement_input.system())
        .add_system(despawning.system().before(Labels::Moving))
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(config.tick_length))
                .with_system(snake_movement.system().label(Labels::Moving))
                .with_system(
                    snake_respawn
                        .system()
                        .label(Labels::Respawning)
                        .after(Labels::Moving),
                )
                .with_system(snake_eating.system().after(Labels::Moving))
                .with_system(snake_collision_check.system().after(Labels::Moving)),
        )
        .add_system(snake_spawn.system())
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(
                    config.tick_length * config.food_ticks as f64,
                ))
                .with_system(food_spawn.system()),
        )
        .add_system_to_stage(CoreStage::PostUpdate, grid_positioning.system())
        .insert_resource(WindowDescriptor {
            title: TITLE.to_string(),
            width: (grid_width * config.grid_scale) as f32,
            height: (grid_height * config.grid_scale) as f32,
            resizable: false,
            ..Default::default()
        })
        .insert_resource(ClearColor(
            Color::hex(&theme.background).unwrap_or(MISSING_COLOR),
        ))
        .insert_resource(Respawn::default())
        .insert_resource(Random::new(&config))
        .insert_non_send_resource(config)
        .insert_resource(theme)
        .insert_resource(GridDimensions {
            width: grid_width,
            height: grid_height,
            scale: grid_scale,
        })
        .init_resource::<DirectionalControls>()
        .add_event::<RespawnEvent>()
        .add_plugins(DefaultPlugins)
        .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    config: NonSend<Config>,
    dimensions: Res<GridDimensions>,
    theme: Res<Theme>,
    mut random: ResMut<Random>,
) {
    commands.insert_resource(AudioAssets::new(&asset_server, &config));

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    let mut wall = |x, y| {
        wall_spawn(
            &mut commands,
            &mut materials,
            GridPosition::new(x, y),
            &dimensions,
            &theme,
        )
    };

    let mut spawn_positions = SpawnPositions::default();
    let mut spawn = |x, y, direction| {
        spawn_positions
            .spawn_positions
            .push(SpawnPosition::new(GridPosition::new(x, y), direction));
    };

    let map_data = config.map.get_map_data(&mut random.environment_generator);
    let top = map_data.height - 1;
    for (x, y, cell) in map_data.iter() {
        match cell {
            Cell::Empty => {}
            Cell::Wall => wall(x, top - y),
            Cell::Spawn(direction) => spawn(x, top - y, direction),
        }
    }

    commands.insert_resource(spawn_positions);
}

fn grid_positioning(
    mut query: Query<(&GridPosition, &mut Transform)>,
    dimensions: Res<GridDimensions>,
) {
    for (grid_position, mut transform) in query.iter_mut() {
        transform.translation = transform.translation.lerp(
            grid_to_vector(grid_position, &dimensions),
            grid_position.t.unwrap_or(1.0),
        );
    }
}

fn grid_to_vector(grid_position: &GridPosition, dimensions: &GridDimensions) -> Vec3 {
    Vec3::new(
        (grid_position.x as f32 - dimensions.width as f32 / 2.0) * dimensions.scale as f32
            + dimensions.scale as f32 / 2.0,
        (grid_position.y as f32 - dimensions.height as f32 / 2.0) * dimensions.scale as f32
            + dimensions.scale as f32 / 2.0,
        0.0,
    )
}

fn food_spawn(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    grid_positions: Query<&GridPosition>,
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
    dimensions: Res<GridDimensions>,
    theme: Res<Theme>,
    mut random: ResMut<Random>,
) {
    // Return and spawn no food if there are no available grid positions (entire grid full)
    if grid_positions.iter().len() >= (dimensions.width * dimensions.height) as usize {
        return;
    }
    // This will prevent an infinite loop here:
    let grid_position = 'outer: loop {
        let possible_grid_position = GridPosition::new(
            random.snake_spawn_generator.next_u32() % dimensions.width,
            random.snake_spawn_generator.next_u32() % dimensions.height,
        );
        for exisiting_grid_position in grid_positions.iter() {
            if exisiting_grid_position.x == possible_grid_position.x
                && exisiting_grid_position.y == possible_grid_position.y
            {
                continue 'outer;
            }
        }
        break possible_grid_position;
    };
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(
                Color::hex(theme.food.choose(&mut random.food_spawn_generator).unwrap())
                    .unwrap_or(MISSING_COLOR)
                    .into(),
            ),
            sprite: Sprite::new(Vec2::new(
                dimensions.scale as f32 * 0.875,
                dimensions.scale as f32 * 0.875,
            )),
            transform: Transform::from_translation(grid_to_vector(&grid_position, &dimensions)),
            ..Default::default()
        })
        .insert(grid_position)
        .insert(Food);
    audio.play(audio_assets.spawn_food.clone_weak());
}

fn wall_spawn(
    commands: &mut Commands,
    materials: &mut Assets<ColorMaterial>,
    grid_position: GridPosition,
    dimensions: &GridDimensions,
    theme: &Theme,
) {
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::hex(&theme.walls).unwrap_or(MISSING_COLOR).into()),
            sprite: Sprite::new(Vec2::new(dimensions.scale as f32, dimensions.scale as f32)),
            transform: Transform::from_translation(grid_to_vector(&grid_position, dimensions)),
            ..Default::default()
        })
        .insert(grid_position)
        .insert(Collidable);
}

fn snake_respawn(
    mut respawn: ResMut<Respawn>,
    mut respawn_writer: EventWriter<RespawnEvent>,
    time: Res<Time>,
) {
    if respawn.time <= time.seconds_since_startup() && !respawn.completed {
        respawn_writer.send(RespawnEvent);
        respawn.completed = true;
    }
}

fn snake_spawn(
    mut commands: Commands,
    mut spawn_reader: EventReader<RespawnEvent>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut windows: ResMut<Windows>,
    spawn_positions: Res<SpawnPositions>,
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
    config: NonSend<Config>,
    dimensions: Res<GridDimensions>,
    theme: Res<Theme>,
    mut random: ResMut<Random>,
) {
    for _respawn_event in spawn_reader.iter() {
        let spawn_position = spawn_positions
            .spawn_positions
            .choose(&mut random.snake_spawn_generator)
            .unwrap();
        let mut snake_head = SnakeHead::new(spawn_position.direction);
        let snake_head_position = spawn_position.grid_position.clone();
        let segment_direction = snake_head.direction.opposite().vec();
        for i in 1..config.snake_spawn_segments {
            snake_head.spawn_segment(
                None,
                &mut commands,
                &mut materials,
                GridPosition::new(
                    ((segment_direction.x * (i as f32)) + snake_head_position.x as f32) as u32,
                    ((segment_direction.y * (i as f32)) + snake_head_position.y as f32) as u32,
                ),
                &mut windows,
                &config,
                &dimensions,
                &theme,
            )
        }
        commands
            .spawn_bundle(SpriteBundle {
                material: materials.add(Color::hex(&theme.snake).unwrap_or(MISSING_COLOR).into()),
                sprite: Sprite::new(Vec2::new(
                    config.grid_scale as f32 * 0.875,
                    config.grid_scale as f32 * 0.875,
                )),
                transform: Transform::from_translation(grid_to_vector(
                    &snake_head_position,
                    &dimensions,
                )),
                ..Default::default()
            })
            .insert(snake_head_position)
            .insert(snake_head);
        audio.play(audio_assets.spawn_snake.clone_weak());
    }
}

fn snake_movement_input(
    mut keyboard_input_reader: EventReader<KeyboardInput>,
    mut snake_heads: Query<&mut SnakeHead>,
    controls: Res<DirectionalControls>,
) {
    let mut direction = None;

    for event in keyboard_input_reader.iter() {
        if event.state == ElementState::Released {
            continue;
        }

        direction = controls.from_keyboard(event).or(direction);
    }

    if let Some(direction) = direction {
        for mut snake_head in snake_heads.iter_mut() {
            if direction == snake_head.direction.opposite() {
                continue;
            }

            snake_head.next_direction = direction;
        }
    }
}

fn snake_movement(
    mut snake_heads: Query<(&mut SnakeHead, &mut GridPosition)>,
    mut grid_positions: Query<&mut GridPosition, Without<SnakeHead>>,
) {
    for (mut snake_head, mut grid_position) in snake_heads.iter_mut() {
        snake_head.direction = snake_head.next_direction;
        let direction_vector = snake_head.direction.vec();
        snake_head.update_segment_positions(&grid_position, &mut grid_positions);
        grid_position.x = (grid_position.x as f32 + direction_vector.x) as u32;
        grid_position.y = (grid_position.y as f32 + direction_vector.y) as u32;
    }
}

struct Collidable;

fn snake_collision_check(
    mut commands: Commands,
    mut snake_heads: Query<(Entity, &SnakeHead, &GridPosition)>,
    grid_positions: Query<&GridPosition, With<Collidable>>,
    time: Res<Time>,
    mut respawn_event: ResMut<Respawn>,
    audio_assets: Res<AudioAssets>,
    config: NonSend<Config>,
    dimensions: Res<GridDimensions>,
) {
    for (snake_head_entity, snake_head, snake_head_position) in snake_heads.iter_mut() {
        let mut despawn = || {
            snake_head.despawn(
                &mut commands,
                snake_head_entity,
                &time,
                &mut respawn_event,
                &audio_assets,
                &config,
            );
        };
        // It is unnecessary to check if the x- or y-positions are less than 0
        // since this is impossible for the unsigned integers that they are stored in
        if snake_head_position.x >= dimensions.width || snake_head_position.y >= dimensions.height {
            despawn();
        }
        for segment in snake_head.segments.iter() {
            let segment_position = match grid_positions.get(*segment) {
                Ok(position) => position,
                Err(_) => continue,
            };
            if snake_head_position.x == segment_position.x
                && snake_head_position.y == segment_position.y
            {
                despawn();
            }
        }
        for grid_position in grid_positions.iter() {
            if snake_head_position.x == grid_position.x && snake_head_position.y == grid_position.y
            {
                despawn();
            }
        }
    }
}

fn snake_eating(
    mut commands: Commands,
    mut snake_heads: Query<(&mut SnakeHead, &GridPosition)>,
    foods: Query<(Entity, &GridPosition), With<Food>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    mut windows: ResMut<Windows>,
    audio_assets: Res<AudioAssets>,
    config: NonSend<Config>,
    dimensions: Res<GridDimensions>,
    theme: Res<Theme>,
) {
    for (mut snake_head, snake_head_grid_position) in snake_heads.iter_mut() {
        for (food, food_position) in foods.iter() {
            if food_position.x == snake_head_grid_position.x
                && food_position.y == snake_head_grid_position.y
            {
                commands
                    .entity(food)
                    .remove::<Food>()
                    .insert(Despawning::new(
                        time.seconds_since_startup(),
                        0.0,
                        Some(audio_assets.eat.clone_weak()),
                    ));
                snake_head.spawn_segment(
                    Some(0),
                    &mut commands,
                    &mut materials,
                    snake_head_grid_position.clone(),
                    &mut windows,
                    &config,
                    &dimensions,
                    &theme,
                );
            }
        }
    }
}

fn despawning(
    mut commands: Commands,
    mut despawning_objects: Query<(
        Entity,
        &mut Despawning,
        &mut Transform,
        &Handle<ColorMaterial>,
    )>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    audio: Res<Audio>,
) {
    for (entity, mut despawning, mut transform, material_handle) in despawning_objects.iter_mut() {
        if time.seconds_since_startup() - despawning.despawn_time < despawning.animation_delay {
            continue;
        }
        if !despawning.started {
            despawning.started = true;
            if despawning.sound.is_some() {
                if let Some(sound) = despawning.sound.take() {
                    audio.play(sound);
                }
            }
        }
        transform.scale *= 1.125;
        let material = materials.get_mut(material_handle).unwrap();
        let alpha = material.color.a() / 1.5;
        material.color.set_a(alpha);
        // Only despawn if alpha value is 0 when converted to an 8-bit color value
        // One can't check if alpha == 0.0 since this will never happen,
        // and using an arbitrary small value (if alpha < 0.01) isn't precise.
        if (alpha * 255.0) as u32 == 0 {
            commands.entity(entity).despawn();
            continue;
        }
    }
}

#[derive(Default)]
struct Respawn {
    time: f64,
    completed: bool,
}

struct RespawnEvent;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Direction {
    Left,
    Right,
    Down,
    Up,
}

impl Direction {
    fn opposite(&self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Down => Self::Up,
            Self::Up => Self::Down,
        }
    }
    fn vec(&self) -> Vec2 {
        match self {
            Self::Left => Vec2::new(-1.0, 0.0),
            Self::Right => Vec2::new(1.0, 0.0),
            Self::Down => Vec2::new(0.0, -1.0),
            Self::Up => Vec2::new(0.0, 1.0),
        }
    }
}

struct SnakeHead {
    direction: Direction,
    next_direction: Direction,
    segments: Vec<Entity>,
}

impl SnakeHead {
    fn new(direction: Direction) -> Self {
        SnakeHead {
            direction,
            next_direction: direction,
            segments: Vec::new(),
        }
    }
    fn spawn_segment(
        &mut self,
        index: Option<usize>,
        commands: &mut Commands,
        materials: &mut Assets<ColorMaterial>,
        grid_position: GridPosition,
        windows: &mut Windows,
        config: &Config,
        dimensions: &GridDimensions,
        theme: &Theme,
    ) {
        self.segments.insert(
            match index {
                Some(index) => index,
                None => self.segments.len(),
            },
            commands
                .spawn_bundle(SpriteBundle {
                    material: materials
                        .add(Color::hex(&theme.snake).unwrap_or(MISSING_COLOR).into()),
                    sprite: Sprite::new(Vec2::new(
                        config.grid_scale as f32 * 0.75,
                        config.grid_scale as f32 * 0.75,
                    )),
                    transform: Transform::from_translation(grid_to_vector(
                        &grid_position,
                        dimensions,
                    )),
                    ..Default::default()
                })
                .insert(SnakeSegment)
                .insert(grid_position)
                .insert(Collidable)
                .id(),
        );
        windows.get_primary_mut().unwrap().set_title(format!(
            "{} — Score: {}",
            TITLE,
            self.segments.len() as u32 + 1 - config.snake_spawn_segments
        ));
    }
    fn update_segment_positions(
        &mut self,
        head_position: &GridPosition,
        grid_positions: &mut Query<&mut GridPosition, Without<SnakeHead>>,
    ) {
        let mut new_segment_positions = Vec::<GridPosition>::new();
        for (i, _segment_position) in self.segments.iter().enumerate() {
            if i == 0 {
                new_segment_positions.push(head_position.clone());
                continue;
            }
            new_segment_positions.push(
                (grid_positions
                    .get_mut(*self.segments.get(i - 1).unwrap())
                    .unwrap())
                .clone(),
            );
        }
        for (i, new_segment_position) in new_segment_positions.iter().enumerate() {
            let mut segment_position = match grid_positions.get_mut(*self.segments.get(i).unwrap())
            {
                Ok(position) => position,
                Err(_) => continue,
            };
            segment_position.x = new_segment_position.x;
            segment_position.y = new_segment_position.y;
        }
    }
    fn despawn(
        &self,
        commands: &mut Commands,
        entity: Entity,
        time: &Time,
        respawn_event: &mut Respawn,
        audio_assets: &AudioAssets,
        config: &Config,
    ) {
        for (i, segment) in self.segments.iter().enumerate() {
            commands
                .entity(*segment)
                .remove::<SnakeSegment>()
                .insert(Despawning::new(
                    time.seconds_since_startup(),
                    (i + 1) as f64 * config.snake_segment_despawn_interval,
                    Some(audio_assets.destroy.clone_weak()),
                ));
        }
        commands
            .entity(entity)
            .remove::<SnakeHead>()
            .insert(Despawning::new(
                time.seconds_since_startup(),
                0.0,
                Some(audio_assets.destroy.clone_weak()),
            ));
        respawn_event.time = time.seconds_since_startup()
            + config.snake_segment_despawn_interval * self.segments.len() as f64
            + config.snake_respawn_delay;
        respawn_event.completed = false;
    }
}

struct SnakeSegment;

struct Despawning {
    despawn_time: f64,
    animation_delay: f64,
    sound: Option<Handle<AudioSource>>,
    started: bool,
}

impl Despawning {
    fn new(despawn_time: f64, animation_delay: f64, sound: Option<Handle<AudioSource>>) -> Self {
        Self {
            despawn_time,
            animation_delay,
            sound,
            started: false,
        }
    }
}

struct Food;

#[derive(Default, Clone)]
struct GridPosition {
    x: u32,
    y: u32,
    t: Option<f32>,
}

impl GridPosition {
    fn new(x: u32, y: u32) -> Self {
        Self {
            x,
            y,
            t: Some(0.375),
        }
    }
}

struct SpawnPosition {
    grid_position: GridPosition,
    direction: Direction,
}

impl SpawnPosition {
    fn new(grid_position: GridPosition, direction: Direction) -> Self {
        SpawnPosition {
            grid_position,
            direction,
        }
    }
}

#[derive(Default)]
struct SpawnPositions {
    spawn_positions: Vec<SpawnPosition>,
}

struct AudioAssets {
    destroy: Handle<AudioSource>,
    eat: Handle<AudioSource>,
    spawn_food: Handle<AudioSource>,
    spawn_snake: Handle<AudioSource>,
}

impl AudioAssets {
    fn new(asset_server: &AssetServer, config: &Config) -> Self {
        let load = |name: &str| asset_server.load(format!("sounds/{}", name).as_str());
        AudioAssets {
            destroy: load(&config.destroy_audio),
            eat: load(&config.eat_audio),
            spawn_food: load(&config.spawn_food_audio),
            spawn_snake: load(&config.spawn_snake_audio),
        }
    }
}
