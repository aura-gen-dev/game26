use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

mod components;

use components::*;

const SCREEN_WIDTH: f32 = 1280.;
const SCREEN_HEIGHT: f32 = 720.;
const PADDLE_WIDTH: f32 = 15.;
const PADDLE_HEIGHT: f32 = 100.;
const PADDLE_PAD: f32 = PADDLE_WIDTH / 2. + 10.;
const BALL_RADIUS: f32 = 8.;
const BALL_SPEED: f32 = 200.;
const PADDLE_SPEED: f32 = 200.;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: (SCREEN_WIDTH, SCREEN_HEIGHT).into(),
                        title: "Pong".to_string(),
                        ..default()
                    }),
                    ..default()
                })
                .build(),
        )
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                player_movement,
                update_position.after(player_movement),
                normalize_ball_speed.before(update_position),
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    // Ball
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Circle {
                radius: BALL_RADIUS,
            })),
            material: materials.add(Color::WHITE),
            ..default()
        },
        Ball,
        Velocity(Vec3::new(BALL_SPEED, BALL_SPEED, 0.0)),
    ));

    // Player
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Rectangle::new(PADDLE_WIDTH, PADDLE_HEIGHT))),
            material: materials.add(Color::WHITE),
            transform: Transform::from_translation(Vec3::new(
                -SCREEN_WIDTH / 2. + PADDLE_PAD,
                0.0,
                0.0,
            )),
            ..default()
        },
        PlayerPaddle,
        Velocity(Vec3::ZERO),
    ));

    // Opponent
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Rectangle::new(15.0, 100.0))),
            material: materials.add(Color::WHITE),
            transform: Transform::from_translation(Vec3::new(
                SCREEN_WIDTH / 2. - PADDLE_PAD,
                0.0,
                0.0,
            )),
            ..default()
        },
        OpponentPaddle,
        Velocity(Vec3::ZERO),
    ));
}

fn update_position(time: Res<Time>, mut query: Query<(&Velocity, &mut Transform)>) {
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation += velocity.0 * time.delta_seconds();
    }
}

fn normalize_ball_speed(mut query: Query<&mut Velocity, With<Ball>>) {
    let mut ball_velocity = query.single_mut();

    if ball_velocity.0.length() != BALL_SPEED {
        ball_velocity.0 = ball_velocity.0.normalize() * BALL_SPEED;
    }
}

fn player_movement(
    key: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Velocity, With<PlayerPaddle>>,
) {
    for mut velocity in query.iter_mut() {
        if key.pressed(KeyCode::KeyW) {
            velocity.0.y = PADDLE_SPEED;
        } else if key.pressed(KeyCode::KeyS) {
            velocity.0.y = -PADDLE_SPEED;
        } else {
            velocity.0.y = 0.;
        }
    }
}
