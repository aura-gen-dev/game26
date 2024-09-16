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
const SCOREBOARD_HEIGHT: f32 = SCREEN_HEIGHT * 0.05;

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
        .insert_resource(Score::default())
        .add_event::<Scored>()
        .add_systems(Startup, (setup, create_scoreboard).chain())
        .add_systems(
            Update,
            (
                player_movement,
                normalize_ball_speed,
                update_position,
                opponent_movement,
                check_collisions,
                check_score_event,
                update_scoreboard,
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
            transform: Transform::from_translation(Vec3::new(
                0.0,
                -SCOREBOARD_HEIGHT,
                0.0,
            )),
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
                -SCOREBOARD_HEIGHT,
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
                -SCOREBOARD_HEIGHT,
                0.0,
            )),
            ..default()
        },
        OpponentPaddle,
    ));

    // Scoreboard "wall"
    commands.spawn(
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Rectangle::new(SCREEN_WIDTH, 1.0))),
            material: materials.add(Color::WHITE),
            transform: Transform::from_translation(Vec3::new(0.0, SCREEN_HEIGHT / 2.0 - SCOREBOARD_HEIGHT, 0.0)),
            ..default()
        },
    );
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
    mut events: EventWriter<Scored>,
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

    if ball_x - BALL_RADIUS <= -SCREEN_WIDTH / 2. {
        // Player scores
        println!("Opponent scores!");
        events.send(Scored(Scorer::Opponent));
        return
    } else if ball_x + BALL_RADIUS >= SCREEN_WIDTH / 2. {
        // Opponent scores
        println!("Player scores!");
        events.send(Scored(Scorer::Player));
        return
    }

    if ball_y - BALL_RADIUS <= -SCREEN_HEIGHT / 2. {
        // Bounce off the bottom wall
        ball_velocity.0.y = -ball_velocity.0.y;
        ball_transform.translation.y = -SCREEN_HEIGHT / 2. + BALL_RADIUS + 1.;
    } else if ball_y + BALL_RADIUS >= SCREEN_HEIGHT / 2. - SCOREBOARD_HEIGHT {
        // Bounce off the top wall
        ball_velocity.0.y = -ball_velocity.0.y;
        ball_transform.translation.y = SCREEN_HEIGHT / 2. - BALL_RADIUS - 1. - SCOREBOARD_HEIGHT;
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

    if player_y + PADDLE_HEIGHT / 2. >= SCREEN_HEIGHT / 2. - SCOREBOARD_HEIGHT {
        // Prevent the player from going off the top of the screen
        player_velocity.0.y = 0.;
        player_transform.translation.y = SCREEN_HEIGHT / 2. - PADDLE_HEIGHT / 2. - SCOREBOARD_HEIGHT;
    } else if player_y - PADDLE_HEIGHT / 2. <= -SCREEN_HEIGHT / 2. {
        // Prevent the player from going off the bottom of the screen
        player_velocity.0.y = 0.;
        player_transform.translation.y = -SCREEN_HEIGHT / 2. + PADDLE_HEIGHT / 2.;
    }

    if opponent_y + PADDLE_HEIGHT / 2. >= SCREEN_HEIGHT / 2. - SCOREBOARD_HEIGHT {
        // Prevent the opponent from going off the top of the screen
        opponent_transform.translation.y = SCREEN_HEIGHT / 2. - PADDLE_HEIGHT / 2. - SCOREBOARD_HEIGHT;
    } else if opponent_y - PADDLE_HEIGHT / 2. <= -SCREEN_HEIGHT / 2. {
        // Prevent the opponent from going off the bottom of the screen
        opponent_transform.translation.y = -SCREEN_HEIGHT / 2. + PADDLE_HEIGHT / 2.;
    }
}

fn check_score_event(
    mut ball: Query<(&mut Transform, &mut Velocity), With<Ball>>,
    mut score: ResMut<Score>,
    mut events: EventReader<Scored>,
) {
    let (mut ball_transform, mut ball_velocity) = ball.single_mut();

    for event in events.read() {
        match event.0 {
            Scorer::Player => {
                score.player += 1;
                ball_transform.translation = Vec3::ZERO;
                ball_velocity.0 = Vec3::new(-BALL_SPEED, BALL_SPEED, 0.0);
            }
            Scorer::Opponent => {
                score.opponent += 1;
                ball_transform.translation = Vec3::ZERO;
                ball_velocity.0 = Vec3::new(BALL_SPEED, -BALL_SPEED, 0.0);
            }
        }
        println!("Score: {} - {}", score.player, score.opponent);
    }
}

fn create_scoreboard(
    mut commands: Commands,
) {
    let score_root = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(40.0),
                height: Val::Percent(5.0),
                position_type: PositionType::Absolute,
                left: Val::Percent(30.0),
                top: Val::Px(0.0),
                ..default()
            },
            background_color: BackgroundColor(Color::WHITE),
            ..default()
        }).id();

    let player_score_box = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(50.0),
                height: Val::Percent(100.0),
                border: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            background_color: BackgroundColor(Color::srgb(0.65, 0.65, 0.65)),
            ..default()
        }).id();

    let player_score_text = commands
        .spawn(TextBundle {
            style: Style {
                height: Val::Percent(100.0),
                margin: UiRect {
                    left: Val::Auto,
                    right: Val::Auto,
                    top: Val::Auto,
                    bottom: Val::Auto,
                },
                ..default()
            },
            text: Text::from_section(
                "Player: 0".to_string(), 
                TextStyle {
                    font_size: 25.0,
                    ..default()
                }
            ),
            ..default()
        })
        .insert(PlayerScoreboard)
        .id();

    let opponent_score_box = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(50.0),
                height: Val::Percent(100.0),
                border: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            background_color: BackgroundColor(Color::srgb(0.65, 0.65, 0.65)),
            ..default()
        }).id();

        let opponent_score_text = commands
            .spawn(TextBundle {
                style: Style {
                    height: Val::Percent(100.0),
                    margin: UiRect {
                        left: Val::Auto,
                        right: Val::Auto,
                        top: Val::Auto,
                        bottom: Val::Auto,
                    },
                    ..default()
                },
                text: Text::from_section(
                    "Opponent: 0".to_string(), 
                    TextStyle {
                        font_size: 25.0,
                        ..default()
                    }
                ),
                ..default()
            })
            .insert(OpponentScoreboard)
            .id();
        
    commands.entity(player_score_box).add_child(player_score_text);
    commands.entity(opponent_score_box).add_child(opponent_score_text);
    commands.entity(score_root).add_child(player_score_box);
    commands.entity(score_root).add_child(opponent_score_box);
}

fn update_scoreboard(
    mut player_scoreboard: Query<&mut Text, With<PlayerScoreboard>>,
    mut opponent_scoreboard: Query<&mut Text, (With<OpponentScoreboard>, Without<PlayerScoreboard>)>,
    score: Res<Score>,
) {
    if score.is_changed() {
        let player_text = format!("Player: {}", score.player);
        let opponent_text = format!("Opponent: {}", score.opponent);

        for mut text in player_scoreboard.iter_mut() {
            text.sections[0].value = player_text.clone();
        }
        for mut text in opponent_scoreboard.iter_mut() {
            text.sections[0].value = opponent_text.clone();
        }
    }
}
