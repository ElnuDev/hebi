#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::core::FixedTimestep;
use bevy::prelude::*;
use rand::prelude::*;
use rand::seq::SliceRandom;
use serde::Deserialize;
use std::fs;

const TITLE: &str = "Hebi";

#[derive(SystemLabel, Debug, Hash, PartialEq, Eq, Clone)]
enum Labels {
    Moving,
    Respawning,
}

#[derive(Deserialize)]
struct Config {
    visuals: ConfigVisuals,
    game: ConfigGame,
    audio: ConfigAudio,
}

#[derive(Deserialize)]
struct ConfigVisuals {
    theme: String,
    scale: u32,
}

#[derive(Deserialize)]
struct ConfigGame {
    grid: ConfigGameGrid,
    tick: ConfigGameTick,
    snake: ConfigGameSnake,
}

#[derive(Deserialize)]
struct ConfigGameGrid {
    width: u32,
    height: u32,
    corner_walls: bool,
}

#[derive(Deserialize)]
struct ConfigGameTick {
    length: f64,
    food: u32,
}

#[derive(Deserialize)]
struct ConfigGameSnake {
    segments: u32,
    segment_despawn_interval: f64,
    respawn_delay: f64,
}

#[derive(Deserialize)]
struct ConfigAudio {
    eat: String,
    destroy: String,
    spawn_food: String,
    spawn_snake: String,
}

#[derive(Deserialize)]
struct Theme {
    walls: String,
    background: String,
    snake: String,
    food: Vec<String>,
}

fn main() {
    let config: Config = toml::from_str(
        &fs::read_to_string("config.toml").expect("Something went wrong reading the config file!"),
    )
    .expect("Something went wrong parsing the config file!");
    let theme: Theme = toml::from_str(
        &fs::read_to_string(format!("themes/{}.toml", config.visuals.theme))
            .expect("Something went wrong reading the theme file!"),
    )
    .expect("Something went wrong parsing the theme file!");

    App::build()
        .add_startup_system(setup.system())
        .add_system(snake_movement_input.system())
        .add_system(despawning.system().before(Labels::Moving))
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(config.game.tick.length))
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
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(
                    config.game.tick.length * config.game.tick.food as f64,
                ))
                .with_system(food_spawn.system()),
        )
        .add_system_to_stage(CoreStage::PostUpdate, grid_positioning.system())
        .insert_resource(WindowDescriptor {
            title: TITLE.to_string(),
            width: (config.game.grid.width * config.visuals.scale) as f32,
            height: (config.game.grid.height * config.visuals.scale) as f32,
            resizable: false,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::hex(&theme.background).unwrap()))
        .insert_resource(RespawnEvent::default())
        .insert_resource(config)
        .insert_resource(theme)
        .add_plugins(DefaultPlugins)
        .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    config: Res<Config>,
    theme: Res<Theme>,
) {
    commands.insert_resource(AudioAssets::new(&asset_server, &config));

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    let mut wall = |x, y| {
        wall_spawn(
            &mut commands,
            &mut materials,
            GridPosition::new(x, y),
            &config,
            &theme,
        )
    };
    for x in 0..config.game.grid.width {
        wall(x, 0);
        wall(x, config.game.grid.height - 1);
    }
    for y in 1..config.game.grid.height - 1 {
        wall(0, y);
        wall(config.game.grid.width - 1, y);
    }

    if config.game.grid.corner_walls {
        // Bottom-left wall block
        wall(2, 2);
        wall(3, 2);
        wall(2, 3);
        wall(3, 3);

        // Top-left wall block
        wall(2, config.game.grid.height - 4);
        wall(3, config.game.grid.height - 4);
        wall(2, config.game.grid.height - 3);
        wall(3, config.game.grid.height - 3);

        // Bottom-right wall block
        wall(config.game.grid.width - 4, 2);
        wall(config.game.grid.width - 3, 2);
        wall(config.game.grid.width - 4, 3);
        wall(config.game.grid.width - 3, 3);

        // Top-right wall block
        wall(config.game.grid.width - 4, config.game.grid.height - 4);
        wall(config.game.grid.width - 3, config.game.grid.height - 4);
        wall(config.game.grid.width - 4, config.game.grid.height - 3);
        wall(config.game.grid.width - 3, config.game.grid.height - 3);
    }

    let mut spawn_positions = SpawnPositions::default();
    let mut spawn = |x, y, direction| {
        spawn_positions
            .spawn_positions
            .push(SpawnPosition::new(GridPosition::new(x, y), direction));
    };

    // Bottom-left spawn
    spawn(5, 5, Direction::Right);
    spawn(5, 5, Direction::Up);

    // Top-left spawn
    spawn(5, config.game.grid.height - 6, Direction::Right);
    spawn(5, config.game.grid.height - 6, Direction::Down);

    // Bottom-right spaw
    spawn(config.game.grid.width - 6, 5, Direction::Left);
    spawn(config.game.grid.width - 6, 5, Direction::Up);

    // Top-right spawn
    spawn(
        config.game.grid.width - 6,
        config.game.grid.height - 6,
        Direction::Left,
    );
    spawn(
        config.game.grid.width - 6,
        config.game.grid.height - 6,
        Direction::Down,
    );

    commands.insert_resource(spawn_positions);
}

fn grid_positioning(mut query: Query<(&GridPosition, &mut Transform)>, config: Res<Config>) {
    for (grid_position, mut transform) in query.iter_mut() {
        transform.translation = transform.translation.lerp(
            grid_to_vector(grid_position, &config),
            grid_position.t.unwrap_or(1.0),
        );
    }
}

fn grid_to_vector(grid_position: &GridPosition, config: &Res<Config>) -> Vec3 {
    Vec3::new(
        (grid_position.x as f32 - config.game.grid.width as f32 / 2.0)
            * config.visuals.scale as f32
            + config.visuals.scale as f32 / 2.0,
        (grid_position.y as f32 - config.game.grid.height as f32 / 2.0)
            * config.visuals.scale as f32
            + config.visuals.scale as f32 / 2.0,
        0.0,
    )
}

fn food_spawn(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    grid_positions: Query<&GridPosition>,
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
    config: Res<Config>,
    theme: Res<Theme>,
) {
    // Return and spawn no food if there are no available grid positions (entire grid full)
    if grid_positions.iter().len() >= (config.game.grid.width * config.game.grid.height) as usize {
        return;
    }
    // This will prevent an infinite loop here:
    let grid_position = 'outer: loop {
        let possible_grid_position = GridPosition::new(
            (random::<f32>() * config.game.grid.width as f32) as u32,
            (random::<f32>() * config.game.grid.height as f32) as u32,
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
                Color::hex(theme.food.choose(&mut rand::thread_rng()).unwrap())
                    .unwrap()
                    .into(),
            ),
            sprite: Sprite::new(Vec2::new(
                config.visuals.scale as f32 * 0.875,
                config.visuals.scale as f32 * 0.875,
            )),
            transform: Transform::from_translation(grid_to_vector(&grid_position, &config)),
            ..Default::default()
        })
        .insert(grid_position)
        .insert(Food);
    audio.play(audio_assets.spawn_food.clone_weak());
}

fn wall_spawn(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    grid_position: GridPosition,
    config: &Res<Config>,
    theme: &Res<Theme>,
) {
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::hex(&theme.walls).unwrap().into()),
            sprite: Sprite::new(Vec2::new(
                config.visuals.scale as f32,
                config.visuals.scale as f32,
            )),
            transform: Transform::from_translation(grid_to_vector(&grid_position, &config)),
            ..Default::default()
        })
        .insert(grid_position)
        .insert(Collidable);
}

fn snake_respawn(
    commands: Commands,
    materials: ResMut<Assets<ColorMaterial>>,
    mut respawn: ResMut<RespawnEvent>,
    time: Res<Time>,
    windows: ResMut<Windows>,
    spawn_positions: Res<SpawnPositions>,
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
    config: Res<Config>,
    theme: Res<Theme>,
) {
    if respawn.time <= time.seconds_since_startup() && !respawn.completed {
        snake_spawn(
            commands,
            materials,
            windows,
            spawn_positions,
            audio,
            audio_assets,
            config,
            theme,
        );
        respawn.completed = true;
    }
}

fn snake_spawn(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut windows: ResMut<Windows>,
    spawn_positions: Res<SpawnPositions>,
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
    config: Res<Config>,
    theme: Res<Theme>,
) {
    let spawn_position = spawn_positions
        .spawn_positions
        .choose(&mut rand::thread_rng())
        .unwrap();
    let mut snake_head = SnakeHead::new(spawn_position.direction);
    let snake_head_position = spawn_position.grid_position.clone();
    let segment_direction = snake_head.direction.opposite().vec();
    for i in 1..config.game.snake.segments {
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
            &theme,
        )
    }
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::hex(&theme.snake).unwrap().into()),
            sprite: Sprite::new(Vec2::new(
                config.visuals.scale as f32 * 0.875,
                config.visuals.scale as f32 * 0.875,
            )),
            transform: Transform::from_translation(grid_to_vector(&snake_head_position, &config)),
            ..Default::default()
        })
        .insert(snake_head_position)
        .insert(snake_head);
    audio.play(audio_assets.spawn_snake.clone_weak());
}

fn snake_movement_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut snake_heads: Query<&mut SnakeHead>,
) {
    for mut snake_head in snake_heads.iter_mut() {
        let direction: Direction = if keyboard_input.pressed(KeyCode::Left) {
            Direction::Left
        } else if keyboard_input.pressed(KeyCode::Down) {
            Direction::Down
        } else if keyboard_input.pressed(KeyCode::Up) {
            Direction::Up
        } else if keyboard_input.pressed(KeyCode::Right) {
            Direction::Right
        } else {
            snake_head.next_direction
        };
        if direction != snake_head.direction.opposite() {
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
    mut respawn_event: ResMut<RespawnEvent>,
    audio_assets: Res<AudioAssets>,
    config: Res<Config>,
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
        if snake_head_position.x >= config.game.grid.width
            || snake_head_position.y >= config.game.grid.height
        {
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
    config: Res<Config>,
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
struct RespawnEvent {
    time: f64,
    completed: bool,
}

#[derive(PartialEq, Copy, Clone)]
enum Direction {
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
        materials: &mut ResMut<Assets<ColorMaterial>>,
        grid_position: GridPosition,
        windows: &mut ResMut<Windows>,
        config: &Res<Config>,
        theme: &Res<Theme>,
    ) {
        self.segments.insert(
            match index {
                Some(index) => index,
                None => self.segments.len(),
            },
            commands
                .spawn_bundle(SpriteBundle {
                    material: materials.add(Color::hex(&theme.snake).unwrap().into()),
                    sprite: Sprite::new(Vec2::new(
                        config.visuals.scale as f32 * 0.75,
                        config.visuals.scale as f32 * 0.75,
                    )),
                    transform: Transform::from_translation(grid_to_vector(&grid_position, config)),
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
            self.segments.len() as u32 + 1 - config.game.snake.segments
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
        time: &Res<Time>,
        respawn_event: &mut ResMut<RespawnEvent>,
        audio_assets: &Res<AudioAssets>,
        config: &Res<Config>,
    ) {
        for (i, segment) in self.segments.iter().enumerate() {
            commands
                .entity(*segment)
                .remove::<SnakeSegment>()
                .insert(Despawning::new(
                    time.seconds_since_startup(),
                    (i + 1) as f64 * config.game.snake.segment_despawn_interval,
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
            + config.game.snake.segment_despawn_interval * self.segments.len() as f64
            + config.game.snake.respawn_delay;
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
    fn new(asset_server: &AssetServer, config: &Res<Config>) -> Self {
        let load = |name: &str| asset_server.load(format!("sounds/{}", name).as_str());
        AudioAssets {
            destroy: load(&config.audio.destroy),
            eat: load(&config.audio.eat),
            spawn_food: load(&config.audio.spawn_food),
            spawn_snake: load(&config.audio.spawn_snake),
        }
    }
}
