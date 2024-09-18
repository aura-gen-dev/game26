use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

mod components;

use components::*;

const SCREEN_WIDTH: f32 = 1280.;
const SCREEN_HEIGHT: f32 = 720.;
const PADDLE_WIDTH: f32 = 100.;
const PADDLE_HEIGHT: f32 = 10.;
const PADDLE_PAD: f32 = 30.;
const PADDLE_SPEED: f32 = 500.;
const BALL_RADIUS: f32 = 8.;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: (SCREEN_WIDTH, SCREEN_HEIGHT).into(),
                        title: "Brick Breaker".to_string(),
                        ..default()
                    }),
                    ..default()
                })
                .build(),
        )
        .add_systems(Startup, setup)
        .add_systems(Update, (player_input, update_position, check_wall_collisions).chain())
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    // Paddle
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Rectangle::new(PADDLE_WIDTH, PADDLE_HEIGHT))),
            material: materials.add(Color::WHITE),
            transform: Transform::from_translation(Vec3::new(
                0.0,
                -SCREEN_HEIGHT / 2. + PADDLE_PAD,
                0.0
            )),
            ..default()
        },
        Paddle,
        Velocity(Vec3::new(0.0, 0.0, 0.0))
    ));

    // Ball
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Circle {
                radius: BALL_RADIUS,
            })),
            material: materials.add(Color::WHITE),
            transform: Transform::from_translation(Vec3::new(
                0.0,
                -SCREEN_HEIGHT / 2. + PADDLE_PAD + PADDLE_HEIGHT + BALL_RADIUS/2.,
                0.0
            )),
            ..default()
        },
        Ball,
        Velocity(Vec3::new(0.0, 0.0, 0.0))
    ));
}

fn player_input(
    key: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Velocity, With<Paddle>>,
) {
    for mut velocity in query.iter_mut() {
        if key.pressed(KeyCode::KeyA) {
            velocity.0.x = -PADDLE_SPEED;
        } else if key.pressed(KeyCode::KeyD) {
            velocity.0.x = PADDLE_SPEED;
        } else {
            velocity.0.x = 0.;
        }
    }
}

fn update_position(time: Res<Time>, mut query: Query<(&Velocity, &mut Transform)>) {
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation += velocity.0 * time.delta_seconds();
    }
}

fn check_wall_collisions(
    mut ball_query: Query<(&mut Velocity, &mut Transform), (With<Ball>, Without<Paddle>)>,
    mut paddle_query: Query<(&mut Velocity, &mut Transform), (With<Paddle>, Without<Ball>)>,
) {
    let (mut ball_velocity, mut ball_transform) = ball_query.single_mut();
    let (mut paddle_velocity, mut paddle_transform) = paddle_query.single_mut();

    if paddle_transform.translation.x - PADDLE_WIDTH/2. < -SCREEN_WIDTH/2. {
        // Clamp left wall
        paddle_velocity.0.x = 0.;
        paddle_transform.translation.x = -SCREEN_WIDTH/2. + PADDLE_WIDTH/2. + 1.
    }
    if paddle_transform.translation.x + PADDLE_WIDTH/2. > SCREEN_WIDTH/2. {
        // Clamp right wall
        paddle_velocity.0.x = 0.;
        paddle_transform.translation.x = SCREEN_WIDTH/2. - PADDLE_WIDTH/2. - 1.
    }
}