use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_math::primitives;
use bevy_rapier2d::{
    control::{KinematicCharacterController, KinematicCharacterControllerOutput},
    dynamics::RigidBody,
    geometry::Collider,
    prelude::{NoUserData, RapierDebugRenderPlugin, RapierPhysicsPlugin},
};

const WINDOW_WIDTH: f32 = 1024.0;
const WINDOW_HEIGHT: f32 = 720.0;
const WINDOW_BOTTOM_Y: f32 = WINDOW_HEIGHT / -2.0;
const WINDOW_LEFT_X: f32 = WINDOW_WIDTH / -2.0;

const COLOR_BACKGROUND: Color = Color::rgb(0.29, 0.31, 0.41);
const COLOR_PLATFORM: Color = Color::rgb(0.13, 0.13, 0.23);
const COLOR_PLAYER: Color = Color::rgb(0.60, 0.55, 0.60);
const COLOR_FLOOR: Color = Color::rgb(0.45, 0.55, 0.66);

const PLAYER_VELOCITY_X: f32 = 400.0;
const PLAYER_VELOCITY_Y: f32 = 850.0;
const MAX_JUMP_HEIGHT: f32 = 230.0;

const FLOOR_THICKNESS: f32 = 10.0;

fn main() {
    App::new()
        .insert_resource(ClearColor(COLOR_BACKGROUND))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bouncy".to_string(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                resizable: false,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(200.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, movement)
        .add_systems(Update, jump)
        .add_systems(Update, do_jump)
        .add_systems(Update, fall)
        .run();
}

fn create_player(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) -> MaterialMesh2dBundle<ColorMaterial> {
    MaterialMesh2dBundle {
        material: materials.add(ColorMaterial::from(COLOR_PLAYER)),
        mesh: meshes.add(primitives::Circle::default()).into(),
        transform: Transform {
            translation: Vec3::new(WINDOW_LEFT_X + 100.0, WINDOW_BOTTOM_Y + 30.0, 0.0),
            scale: Vec3::new(30.0, 30.0, 1.0),
            ..Default::default()
        },
        ..default()
    }
}

fn setup(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());
    commands
        .spawn(create_player(meshes, materials))
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(0.5))
        .insert(KinematicCharacterController::default());

    commands.spawn(Platform::create(-100.0, 200.0, Vec3::new(75.0, 20.0, 1.0)));
    commands.spawn(Platform::create(100.0, 550.0, Vec3::new(100.0, 20.0, 1.0)));
    commands.spawn(Platform::create(50.0, 350.0, Vec3::new(150.0, 20.0, 1.0)));

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: COLOR_FLOOR,
                ..Default::default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, WINDOW_BOTTOM_Y + (FLOOR_THICKNESS / 2.0), 0.0),
                scale: Vec3::new(WINDOW_WIDTH, FLOOR_THICKNESS, 1.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(0.5, 0.5));
}

#[derive(Bundle)]
struct Platform {
    sprite: SpriteBundle,
    rigid_body: RigidBody,
    collider: Collider,
}
impl Platform {
    fn create(x: f32, y: f32, scale: Vec3) -> Self {
        Self {
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: COLOR_PLATFORM,
                    ..Default::default()
                },
                transform: Transform {
                    translation: Vec3::new(x, WINDOW_BOTTOM_Y + FLOOR_THICKNESS + y, 0.0),
                    scale,
                    ..Default::default()
                },
                ..Default::default()
            },
            rigid_body: RigidBody::Fixed,
            collider: Collider::cuboid(0.5, 0.5),
        }
    }
}
// --- Player Controls

#[derive(Component)]
struct Jump(f32);

fn jump(
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    query: Query<
        (Entity, &KinematicCharacterControllerOutput),
        (With<KinematicCharacterController>, Without<Jump>),
    >,
) {
    if query.is_empty() {
        return;
    }

    let (player, output) = query.single();

    if input.just_pressed(KeyCode::Space) && output.grounded {
        commands.entity(player).insert(Jump(0.0));
    }
}

fn movement(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut KinematicCharacterController>,
) {
    let mut player = query.single_mut();
    let mut movement = 0.0;

    if input.pressed(KeyCode::ArrowRight) {
        movement += time.delta_seconds() * PLAYER_VELOCITY_X;
    }
    if input.pressed(KeyCode::ArrowLeft) {
        movement += time.delta_seconds() * PLAYER_VELOCITY_X * -1.0;
    }

    match player.translation {
        Some(vec) => player.translation = Some(Vec2::new(movement, vec.y)),
        None => player.translation = Some(Vec2::new(movement, 0.0)),
    }
}

fn do_jump(
    mut command: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut KinematicCharacterController, &mut Jump)>,
) {
    if query.is_empty() {
        return;
    }
    let (entity, mut player, mut jump) = query.single_mut();

    let mut movement = time.delta().as_secs_f32() * PLAYER_VELOCITY_Y;
    if movement + jump.0 >= MAX_JUMP_HEIGHT {
        movement = MAX_JUMP_HEIGHT - jump.0;
        command.entity(entity).remove::<Jump>();
    }
    jump.0 += movement;
    match player.translation {
        Some(vec) => player.translation = Some(Vec2::new(vec.x, movement)),
        None => player.translation = Some(Vec2::new(0.0, movement)),
    }
}

fn fall(time: Res<Time>, mut query: Query<&mut KinematicCharacterController, Without<Jump>>) {
    if query.is_empty() {
        return;
    }
    let mut player = query.single_mut();
    let movement = time.delta().as_secs_f32() * (PLAYER_VELOCITY_Y / 1.5) * -1.0;

    match player.translation {
        Some(vec) => player.translation = Some(Vec2::new(vec.x, movement)),
        None => player.translation = Some(Vec2::new(0.0, movement)),
    }
}
