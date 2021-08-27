#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::core::FixedTimestep;
use bevy::prelude::*;
use rand::prelude::*;
use rand::seq::SliceRandom;

#[allow(unused)]
mod colors;
#[allow(unused)]
mod themes;

use themes::dracula as theme;

// World width in grid cells
const GRID_WIDTH: u32 = 10;

// World height in grid cells
const GRID_HEIGHT: u32 = 10;

// Pixel dimension of grid cell
const GRID_SCALE: u32 = 36;

const TITLE: &str = "Hebi";

#[derive(SystemLabel, Debug, Hash, PartialEq, Eq, Clone)]
enum Labels {
    Moving,
    Respawning,
}

fn main() {
    const TICK_LENGTH: f64 = 0.2;
    App::build()
        .add_startup_system(setup.system())
        .add_startup_stage("world_spawn", SystemStage::single(world_spawn.system()))
        .add_system(snake_movement_input.system())
        .add_system(despawning.system().before(Labels::Moving))
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TICK_LENGTH))
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
                .with_run_criteria(FixedTimestep::step(TICK_LENGTH * 16.0))
                .with_system(food_spawn.system()),
        )
        .add_system_to_stage(CoreStage::PostUpdate, grid_positioning.system())
        .insert_resource(WindowDescriptor {
            title: TITLE.to_string(),
            width: (GRID_WIDTH * GRID_SCALE) as f32,
            height: (GRID_HEIGHT * GRID_SCALE) as f32,
            resizable: false,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::hex(theme::GRID_BACKGROUND).unwrap()))
        .insert_resource(RespawnEvent::default())
        .add_plugins(DefaultPlugins)
        .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(AudioAssets::new(&asset_server));

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    let mut wall = |x, y| wall_spawn(&mut commands, &mut materials, GridPosition::new(x, y));
    for x in 0..GRID_WIDTH {
        wall(x, 0);
        wall(x, GRID_HEIGHT - 1);
    }
    for y in 1..GRID_HEIGHT - 1 {
        wall(0, y);
        wall(GRID_WIDTH - 1, y);
    }

    const CORNER_WALLS: bool = true;

    if CORNER_WALLS {
        // Bottom-left wall block
        wall(2, 2);
        wall(3, 2);
        wall(2, 3);
        wall(3, 3);
    
        // Top-left wall block
        wall(2, GRID_HEIGHT - 4);
        wall(3, GRID_HEIGHT - 4);
        wall(2, GRID_HEIGHT - 3);
        wall(3, GRID_HEIGHT - 3);
    
        // Bottom-right wall block
        wall(GRID_WIDTH - 4, 2);
        wall(GRID_WIDTH - 3, 2);
        wall(GRID_WIDTH - 4, 3);
        wall(GRID_WIDTH - 3, 3);
    
        // Top-right wall block
        wall(GRID_WIDTH - 4, GRID_HEIGHT - 4);
        wall(GRID_WIDTH - 3, GRID_HEIGHT - 4);
        wall(GRID_WIDTH - 4, GRID_HEIGHT - 3);
        wall(GRID_WIDTH - 3, GRID_HEIGHT - 3);
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
    spawn(5, GRID_HEIGHT - 6, Direction::Right);
    spawn(5, GRID_HEIGHT - 6, Direction::Down);

    // Bottom-right spaw
    spawn(GRID_WIDTH - 6, 5, Direction::Left);
    spawn(GRID_WIDTH - 6, 5, Direction::Up);

    // Top-right spawn
    spawn(GRID_WIDTH - 6, GRID_HEIGHT - 6, Direction::Left);
    spawn(GRID_WIDTH - 6, GRID_HEIGHT - 6, Direction::Down);

    commands.insert_resource(spawn_positions);
}

fn grid_positioning(mut query: Query<(&GridPosition, &mut Transform)>) {
    for (grid_position, mut transform) in query.iter_mut() {
        transform.translation = transform.translation.lerp(
            grid_to_vector(grid_position),
            grid_position.t.unwrap_or(1.0),
        );
    }
}

fn grid_to_vector(grid_position: &GridPosition) -> Vec3 {
    Vec3::new(
        (grid_position.x as f32 - GRID_WIDTH as f32 / 2.0) * GRID_SCALE as f32
            + GRID_SCALE as f32 / 2.0,
        (grid_position.y as f32 - GRID_HEIGHT as f32 / 2.0) * GRID_SCALE as f32
            + GRID_SCALE as f32 / 2.0,
        0.0,
    )
}

fn world_spawn(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn_bundle(SpriteBundle {
        material: materials.add(Color::hex(theme::GRID_BACKGROUND).unwrap().into()),
        sprite: Sprite::new(Vec2::new(
            (GRID_WIDTH * GRID_SCALE) as f32,
            (GRID_HEIGHT * GRID_SCALE) as f32,
        )),
        ..Default::default()
    });
}

fn food_spawn(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    grid_positions: Query<&GridPosition>,
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
) {
    // Return and spawn no food if there are no available grid positions (entire grid full)
    if grid_positions.iter().len() >= (GRID_WIDTH * GRID_HEIGHT) as usize {
        return;
    }
    // This will prevent an infinite loop here:
    let grid_position = 'outer: loop {
        let possible_grid_position = GridPosition::random();
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
                Color::hex(theme::FOOD.choose(&mut rand::thread_rng()).unwrap())
                    .unwrap()
                    .into(),
            ),
            sprite: Sprite::new(Vec2::new(
                GRID_SCALE as f32 * 0.875,
                GRID_SCALE as f32 * 0.875,
            )),
            transform: Transform::from_translation(grid_to_vector(&grid_position)),
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
) {
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::hex(theme::BACKGROUND).unwrap().into()),
            sprite: Sprite::new(Vec2::new(GRID_SCALE as f32, GRID_SCALE as f32)),
            transform: Transform::from_translation(grid_to_vector(&grid_position)),
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
) {
    if respawn.time <= time.seconds_since_startup() && !respawn.completed {
        snake_spawn(
            commands,
            materials,
            windows,
            spawn_positions,
            audio,
            audio_assets,
        );
        respawn.completed = true;
    }
}

const SPAWN_SNAKE_SEGMENTS: u32 = 2;

fn snake_spawn(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut windows: ResMut<Windows>,
    spawn_positions: Res<SpawnPositions>,
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
) {
    let spawn_position = spawn_positions
        .spawn_positions
        .choose(&mut rand::thread_rng())
        .unwrap();
    let mut snake_head = SnakeHead::new(spawn_position.direction);
    let snake_head_position = spawn_position.grid_position.clone();
    let segment_direction = snake_head.direction.opposite().vec();
    for i in 1..SPAWN_SNAKE_SEGMENTS {
        snake_head.spawn_segment(
            None,
            &mut commands,
            &mut materials,
            GridPosition::new(
                ((segment_direction.x * (i as f32)) + snake_head_position.x as f32) as u32,
                ((segment_direction.y * (i as f32)) + snake_head_position.y as f32) as u32,
            ),
            &mut windows,
        )
    }
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::hex(theme::SNAKE).unwrap().into()),
            sprite: Sprite::new(Vec2::new(
                GRID_SCALE as f32 * 0.875,
                GRID_SCALE as f32 * 0.875,
            )),
            transform: Transform::from_translation(grid_to_vector(&snake_head_position)),
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
) {
    for (snake_head_entity, snake_head, snake_head_position) in snake_heads.iter_mut() {
        let mut despawn = || {
            snake_head.despawn(
                &mut commands,
                snake_head_entity,
                &time,
                &mut respawn_event,
                &audio_assets,
            );
        };
        // It is unnecessary to check if the x- or y-positions are less than 0
        // since this is impossible for the unsigned integers that they are stored in
        if snake_head_position.x >= GRID_WIDTH || snake_head_position.y >= GRID_HEIGHT {
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
    ) {
        self.segments.insert(
            match index {
                Some(index) => index,
                None => self.segments.len(),
            },
            commands
                .spawn_bundle(SpriteBundle {
                    material: materials.add(Color::hex(theme::SNAKE).unwrap().into()),
                    sprite: Sprite::new(Vec2::new(
                        GRID_SCALE as f32 * 0.75,
                        GRID_SCALE as f32 * 0.75,
                    )),
                    transform: Transform::from_translation(grid_to_vector(&grid_position)),
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
            self.segments.len() as u32 + 1 - SPAWN_SNAKE_SEGMENTS
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
    ) {
        const SEGMENT_DESPAWN_INTERVAL: f64 = 0.1;
        const RESPAWN_DELAY: f64 = 0.5;
        for (i, segment) in self.segments.iter().enumerate() {
            commands
                .entity(*segment)
                .remove::<SnakeSegment>()
                .insert(Despawning::new(
                    time.seconds_since_startup(),
                    (i + 1) as f64 * SEGMENT_DESPAWN_INTERVAL,
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
            + SEGMENT_DESPAWN_INTERVAL * self.segments.len() as f64
            + RESPAWN_DELAY;
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
    fn random() -> Self {
        Self::new(
            (random::<f32>() * GRID_WIDTH as f32) as u32,
            (random::<f32>() * GRID_HEIGHT as f32) as u32,
        )
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
    fn new(asset_server: &AssetServer) -> Self {
        let load = |name: &str| asset_server.load(format!("sounds/{}.mp3", name).as_str());
        AudioAssets {
            destroy: load("destroy"),
            eat: load("eat"),
            spawn_food: load("spawn_food"),
            spawn_snake: load("spawn_snake"),
        }
    }
}
