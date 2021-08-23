use bevy::prelude::*;
use bevy::core::FixedTimestep;

#[allow(unused)] mod colors;
#[allow(unused)] mod themes;

use themes::dracula as theme;

// World width in grid cells
const GRID_WIDTH: u32 = 28;

// World height in grid cells
const GRID_HEIGHT: u32 = 28;

// Pixel dimension of grid cell
const GRID_SCALE: u32 = 24;

// Pixel padding outside of grid
const GRID_PADDING: u32 = 48;

fn main() {
    App::build()
        .add_startup_system(setup.system())
        .add_startup_stage("world_spawn", SystemStage::single(world_spawn.system()))
        .add_startup_stage("snake_spawn", SystemStage::single(snake_spawn.system()))
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(0.25))
                .with_system(snake_movement.system())
        )
        .add_system_to_stage(CoreStage::PostUpdate, grid_positioning.system())
        .insert_resource({
            let title = "Hebi".to_string();
            let width = (GRID_WIDTH * GRID_SCALE + GRID_PADDING * 2) as f32;
            let height = (GRID_HEIGHT * GRID_SCALE + GRID_PADDING * 2) as f32;
            println!(
                "Configuring window with a title of '{}', a width of {} pixels, and a height of {} pixels.",
                title, width, height
            );
            WindowDescriptor {
                title,
                width,
                height,
                resizable: false,
                ..Default::default()
            }
        })
        .insert_resource(ClearColor(Color::hex(theme::BACKGROUND).unwrap()))
        .add_plugins(DefaultPlugins)
        .run();
}

fn setup(
    mut commands: Commands,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.insert_resource(Materials::new(materials));
}

fn grid_positioning(
    mut query: Query<(&GridPosition, &mut Transform)>,
) {
    for (grid_position, mut transform) in query.iter_mut() {
        assert!(grid_position.in_bounds());
        transform.translation = Vec3::new(
            (grid_position.x as f32 - GRID_WIDTH as f32 / 2.0) * GRID_SCALE as f32 + GRID_SCALE as f32 / 2.0,
            (grid_position.y as f32 - GRID_HEIGHT as f32 / 2.0) * GRID_SCALE as f32 + GRID_SCALE as f32 / 2.0,
            0.0,
        );
    }
}

fn world_spawn(
    mut commands: Commands,
    materials: Res<Materials>,
) {
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.grid_background.clone(),
            sprite: Sprite::new(
                Vec2::new(
                    (GRID_WIDTH * GRID_SCALE) as f32,
                    (GRID_HEIGHT * GRID_SCALE) as f32
                )
            ),
            ..Default::default()
        });
}

fn snake_spawn(
    mut commands: Commands,
    materials: Res<Materials>,
) {
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.snake_head.clone(),
            sprite: Sprite::new(Vec2::new(GRID_SCALE as f32, GRID_SCALE as f32)),
            ..Default::default()
        })
        .insert(GridPosition { x: 0, y: 0 })
        .insert(SnakeHead);
}

fn snake_movement(
    mut query: Query<&mut GridPosition, With<SnakeHead>>,
) {
    for mut grid_position in query.iter_mut() {
        grid_position.x += 1;
    }
}

struct SnakeHead;

struct GridPosition {
    x: u32,
    y: u32,
}

impl GridPosition {
    fn in_bounds(&self) -> bool {
        self.x < GRID_WIDTH && self.y < GRID_HEIGHT
    }
}

// Materials

struct Materials {
    grid_background: Handle<ColorMaterial>,
    snake_head: Handle<ColorMaterial>,
}

impl Materials {
    fn new(mut materials: ResMut<Assets<ColorMaterial>>) -> Self {
        Materials {
            grid_background: materials.add(Color::hex(theme::GRID_BACKGROUND).unwrap().into()),
            snake_head: materials.add(Color::hex(theme::SNAKE_HEAD).unwrap().into()),
        }
    }
}