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
const BALL_SPEED: f32 = 400.;
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
                normalize_ball_speed,
                update_position,
                opponent_movement,
                check_collisions,
            ).chain(),
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
        Velocity(Vec3::new(-BALL_SPEED, BALL_SPEED, 0.0)),
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

fn opponent_movement(
    ball_query: Query<&Transform, With<Ball>>,
    mut query: Query<&mut Transform, (With<OpponentPaddle>, Without<Ball>)>,
) {
    let ball_transform= ball_query.single();
    let mut paddle_transform = query.single_mut();

    paddle_transform.translation.y = ball_transform.translation.y;
}

fn check_collisions(
    mut ball_query: Query<(&mut Velocity, &mut Transform), (With<Ball>, Without<PlayerPaddle>, Without<OpponentPaddle>)>,
    mut player_query: Query<(&mut Velocity, &mut Transform), (With<PlayerPaddle>, Without<OpponentPaddle>)>,
    mut opponent_query: Query<&mut Transform, (With<OpponentPaddle>, Without<Ball>, Without<PlayerPaddle>)>,
) {
    let (mut ball_velocity, mut ball_transform) = ball_query.single_mut();
    let (mut player_velocity, mut player_transform) = player_query.single_mut();
    let mut opponent_transform = opponent_query.single_mut();

    let ball_x = ball_transform.translation.x;
    let ball_y = ball_transform.translation.y;

    let player_x = player_transform.translation.x;
    let player_y = player_transform.translation.y;

    let opponent_x = opponent_transform.translation.x;
    let opponent_y = opponent_transform.translation.y;

    // if ball_x - BALL_RADIUS <= -SCREEN_WIDTH / 2. {
    //     // Player scores
    //     println!("Player scores!");
    //     return
    // } else if ball_x + BALL_RADIUS >= SCREEN_WIDTH / 2. {
    //     // Opponent scores
    //     println!("Opponent scores!");
    //     return
    // }

    if ball_y - BALL_RADIUS <= -SCREEN_HEIGHT / 2. {
        // Bounce off the bottom wall
        ball_velocity.0.y = -ball_velocity.0.y;
        ball_transform.translation.y = -SCREEN_HEIGHT / 2. + BALL_RADIUS + 1.;
    } else if ball_y + BALL_RADIUS >= SCREEN_HEIGHT / 2. {
        // Bounce off the top wall
        ball_velocity.0.y = -ball_velocity.0.y;
        ball_transform.translation.y = SCREEN_HEIGHT / 2. - BALL_RADIUS - 1.;
    }

    if ball_x - BALL_RADIUS <= player_x + PADDLE_WIDTH / 2.
        && ball_y - BALL_RADIUS <= player_y + PADDLE_HEIGHT / 2.
        && ball_y + BALL_RADIUS >= player_y - PADDLE_HEIGHT / 2.
    {
        // Bounce off the player paddle
        ball_velocity.0.x = -ball_velocity.0.x;
        ball_transform.translation.x = player_x + PADDLE_WIDTH / 2. + BALL_RADIUS + 1.;
    } else if ball_x + BALL_RADIUS >= opponent_x - PADDLE_WIDTH / 2.
        && ball_y - BALL_RADIUS <= opponent_y + PADDLE_HEIGHT / 2.
        && ball_y + BALL_RADIUS >= opponent_y - PADDLE_HEIGHT / 2.
    {
        // Bounce off the opponent paddle
        ball_velocity.0.x = -ball_velocity.0.x;
        ball_transform.translation.x = opponent_x - PADDLE_WIDTH / 2. - BALL_RADIUS - 1.;
    }

    if player_y + PADDLE_HEIGHT / 2. >= SCREEN_HEIGHT / 2. {
        // Prevent the player from going off the top of the screen
        player_velocity.0.y = 0.;
        player_transform.translation.y = SCREEN_HEIGHT / 2. - PADDLE_HEIGHT / 2.;
    } else if player_y - PADDLE_HEIGHT / 2. <= -SCREEN_HEIGHT / 2. {
        // Prevent the player from going off the bottom of the screen
        player_velocity.0.y = 0.;
        player_transform.translation.y = -SCREEN_HEIGHT / 2. + PADDLE_HEIGHT / 2.;
    }

    if opponent_y + PADDLE_HEIGHT / 2. >= SCREEN_HEIGHT / 2. {
        // Prevent the opponent from going off the top of the screen
        opponent_transform.translation.y = SCREEN_HEIGHT / 2. - PADDLE_HEIGHT / 2.;
    } else if opponent_y - PADDLE_HEIGHT / 2. <= -SCREEN_HEIGHT / 2. {
        // Prevent the opponent from going off the bottom of the screen
        opponent_transform.translation.y = -SCREEN_HEIGHT / 2. + PADDLE_HEIGHT / 2.;
    }
}

