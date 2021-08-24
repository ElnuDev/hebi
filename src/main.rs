use bevy::prelude::*;
use bevy::core::FixedTimestep;
use rand::prelude::*;
use rand::seq::SliceRandom;

#[allow(unused)] mod colors;
#[allow(unused)] mod themes;

use themes::dracula as theme;

// World width in grid cells
const GRID_WIDTH: u32 = 11;

// World height in grid cells
const GRID_HEIGHT: u32 = 11;

// Pixel dimension of grid cell
const GRID_SCALE: u32 = 24;

// Pixel padding outside of grid
const GRID_PADDING: u32 = 24;

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
        .add_startup_stage("snake_spawn", SystemStage::single(snake_spawn.system()))
        .add_system(snake_movement_input.system())
        .add_system(despawning.system())
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TICK_LENGTH))
                .with_system(snake_movement.system().label(Labels::Moving))
                .with_system(snake_respawn.system().label(Labels::Respawning).after(Labels::Moving))
                .with_system(snake_eating.system().after(Labels::Moving))
                .with_system(snake_collision_check.system().after(Labels::Moving).before(Labels::Respawning))
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TICK_LENGTH * 16.0))
                .with_system(food_spawn.system())
        )
        .add_system_to_stage(CoreStage::PostUpdate, grid_positioning.system())
        .insert_resource(WindowDescriptor {
            title: "Hebi".to_string(),
            width: (GRID_WIDTH * GRID_SCALE + GRID_PADDING * 2) as f32,
            height: (GRID_HEIGHT * GRID_SCALE + GRID_PADDING * 2) as f32,
            resizable: false,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::hex(theme::BACKGROUND).unwrap()))
        .add_event::<RespawnEvent>()
        .add_plugins(DefaultPlugins)
        .run();
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn grid_positioning(
    mut query: Query<(&GridPosition, &mut Transform)>,
) {
    for (grid_position, mut transform) in query.iter_mut() {
        transform.translation = transform.translation.lerp(
            grid_to_vector(grid_position),
            match grid_position.t {
                Some(t) => t,
                None => 1.0,
            },
        );
    }
}

fn grid_to_vector(grid_position: &GridPosition) -> Vec3 {
    Vec3::new(
        (grid_position.x as f32 - GRID_WIDTH as f32 / 2.0) * GRID_SCALE as f32 + GRID_SCALE as f32 / 2.0,
        (grid_position.y as f32 - GRID_HEIGHT as f32 / 2.0) * GRID_SCALE as f32 + GRID_SCALE as f32 / 2.0,
        0.0,
    )
}

fn world_spawn(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::hex(theme::GRID_BACKGROUND).unwrap().into()),
            sprite: Sprite::new(
                Vec2::new(
                    (GRID_WIDTH * GRID_SCALE) as f32,
                    (GRID_HEIGHT * GRID_SCALE) as f32
                )
            ),
            ..Default::default()
        });
}

fn food_spawn(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let grid_position = GridPosition::random();
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::hex(theme::FOOD.choose(&mut rand::thread_rng()).unwrap()).unwrap().into()),
            sprite: Sprite::new(Vec2::new(GRID_SCALE as f32 * 0.875, GRID_SCALE as f32 * 0.875)),
            transform: Transform::from_translation(grid_to_vector(&grid_position)),
            ..Default::default()
        })
        .insert(grid_position)
        .insert(Food);
}

fn snake_respawn(
    commands: Commands,
    materials: ResMut<Assets<ColorMaterial>>,
    mut respawn_reader: EventReader<RespawnEvent>,
) {
    if respawn_reader.iter().next().is_some() {
        snake_spawn(commands, materials);
    }
}

fn snake_spawn(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    const DIRECTION: Direction = Direction::Up;
    const SEGMENTS: u32 = 2;
    let mut snake_head = SnakeHead::new(DIRECTION);
    let snake_head_position = GridPosition::center();
    let segment_direction = snake_head.direction.opposite().vec();
    for i in 1..SEGMENTS {
        snake_head.spawn_segment(None, &mut commands, &mut materials, GridPosition::new(
            ((segment_direction.x * (i as f32)) + snake_head_position.x as f32) as u32,
            ((segment_direction.y * (i as f32)) + snake_head_position.y as f32) as u32,
        ))
    }
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::hex(theme::SNAKE).unwrap().into()),
            sprite: Sprite::new(Vec2::new(GRID_SCALE as f32 * 0.875, GRID_SCALE as f32 * 0.875)),
            transform: Transform::from_translation(grid_to_vector(&snake_head_position)),
            ..Default::default()
        })
        .insert(snake_head_position)
        .insert(snake_head);
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
    mut commands: Commands,
    mut snake_heads: Query<(Entity, &mut SnakeHead, &mut GridPosition)>,
    mut grid_positions: Query<&mut GridPosition, Without<SnakeHead>>,
    mut respawn_writer: EventWriter<RespawnEvent>,
    time: Res<Time>,
) {
    for (entity, mut snake_head, mut grid_position) in snake_heads.iter_mut() {
        snake_head.direction = snake_head.next_direction;
        let direction_vector = snake_head.direction.vec();
        snake_head.update_segment_positions(&grid_position, &mut grid_positions);
        let float_grid_position_x = grid_position.x as f32 + direction_vector.x;
        let float_grid_position_y = grid_position.y as f32 + direction_vector.y;
        if float_grid_position_x < 0.0 || float_grid_position_x >= GRID_WIDTH as f32 || float_grid_position_y < 0.0 || float_grid_position_y >= GRID_HEIGHT as f32 {
            snake_head.despawn(&mut commands, entity, &time);
            respawn_writer.send(RespawnEvent);
            continue;
        }
        grid_position.x = float_grid_position_x as u32;
        grid_position.y = float_grid_position_y as u32;
    }
}

fn snake_collision_check(
    mut commands: Commands,
    mut snake_heads: Query<(Entity, &SnakeHead, &GridPosition)>,
    grid_positions: Query<&GridPosition>,
    mut respawn_writer: EventWriter<RespawnEvent>,
    time: Res<Time>
) {
    for (snake_head_entity, snake_head, snake_head_position) in snake_heads.iter_mut() {
        for segment in snake_head.segments.iter() {
            let segment_position = match grid_positions.get(*segment) {
                Ok(position) => position,
                Err(_) => continue,
            };
            if segment_position.x == snake_head_position.x && segment_position.y == snake_head_position.y {
                snake_head.despawn(&mut commands, snake_head_entity, &time);
                respawn_writer.send(RespawnEvent);
                break;
            } 
        }
    }
}

fn snake_eating(
    mut commands: Commands,
    mut snake_heads: Query<(&mut SnakeHead, &GridPosition)>,
    foods: Query<(Entity, &GridPosition), With<Food>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>
) {
    for (mut snake_head, snake_head_grid_position) in snake_heads.iter_mut() {
        for (food, food_position) in foods.iter() {
            if food_position.x == snake_head_grid_position.x && food_position.y == snake_head_grid_position.y {
                commands.entity(food)
                    .remove::<Food>()
                    .insert(Despawning(time.seconds_since_startup()));
                snake_head.spawn_segment(Some(0), &mut commands, &mut materials, snake_head_grid_position.clone());
            }
        }
    }
}

fn despawning(
    mut commands: Commands,
    mut despawning_objects: Query<(Entity, &Despawning, &mut Transform, &Handle<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
) {
    for (entity, despawning, mut transform, material_handle) in despawning_objects.iter_mut() {
        if time.seconds_since_startup() - despawning.0 > 1.0 {
            commands.entity(entity).despawn();
            continue;
        }
        transform.scale *= 1.125;
        let material = materials.get_mut(material_handle).unwrap();
        material.color.set_a(material.color.a() / 1.5);
    }
}

struct RespawnEvent;

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
            direction: direction,
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
    ) {
        self.segments.insert(
            match index {
                Some(index) => index,
                None => self.segments.len()
            },
            commands
                .spawn_bundle(SpriteBundle {
                    material: materials.add(Color::hex(theme::SNAKE).unwrap().into()),
                    sprite: Sprite::new(Vec2::new(GRID_SCALE as f32 * 0.75, GRID_SCALE as f32 * 0.75)),
                    transform: Transform::from_translation(grid_to_vector(&grid_position)),
                    ..Default::default()
                })
                .insert(SnakeSegment)
                .insert(grid_position)
                .id()
        );
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
            new_segment_positions.push((grid_positions.get_mut(*self.segments.get(i - 1).unwrap()).unwrap()).clone());
        }
        for (i, new_segment_position) in new_segment_positions.iter().enumerate() {
            let mut segment_position = match grid_positions.get_mut(*self.segments.get(i).unwrap()) {
                Ok(position) => position,
                Err(_) => continue,
            };
            segment_position.x = new_segment_position.x;
            segment_position.y = new_segment_position.y;
        }
    }
    fn despawn(&self, commands: &mut Commands, entity: Entity, time: &Res<Time>) {
        for segment in self.segments.iter() {
            commands.entity(*segment)
                .remove::<SnakeSegment>()
                .insert(Despawning(time.seconds_since_startup()));
        }
        commands.entity(entity)
            .remove::<SnakeHead>()
            .insert(Despawning(time.seconds_since_startup()));
    }
}

struct SnakeSegment;

struct Despawning(f64);

struct Food;

#[derive(Default, Clone)]
struct GridPosition {
    x: u32,
    y: u32,
    t: Option<f32>,
}

impl GridPosition {
    fn new(x: u32, y: u32) -> Self {
        Self { x, y, t: Some(0.375) }
    }
    fn center() -> Self {
        Self::new(
            (GRID_WIDTH as f32 / 2.0) as u32,
            (GRID_HEIGHT as f32 / 2.0) as u32,
        )
    }
    fn random() -> Self {
        Self::new(
            (random::<f32>() * GRID_WIDTH as f32) as u32,
            (random::<f32>() * GRID_WIDTH as f32) as u32,
        )
    }
}