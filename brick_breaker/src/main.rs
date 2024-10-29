use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    math::bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume},
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
const BALL_SPEED: f32 = 400.;
const BRICK_WIDTH: f32 = 100.;
const BRICK_HEIGHT: f32 = 30.;
const BRICK_PAD_LR: f32 = 30.;
const BRICK_PAD_TOP: f32 = 50.0;
const BRICK_PAD_BOTTOM: f32 = 300.0;
const BRICK_SPACE: f32 = 1.0;

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
        .init_state::<GameState>()
        .add_event::<CollisionEvent>()
        .add_systems(Startup, setup)
        .add_systems(Update, (player_input, update_position, check_collisions).chain())
        .add_systems(Update, ball_follow.run_if(in_state(GameState::Start)))
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
            mesh: Mesh2dHandle(meshes.add(Rectangle::new(1.0, 1.0))),
            material: materials.add(Color::WHITE),
            transform: Transform {
                translation: Vec3::new(
                    0.0,
                    -SCREEN_HEIGHT / 2. + PADDLE_PAD,
                    0.0
                ),
                scale: Vec3::new(PADDLE_WIDTH, PADDLE_HEIGHT, 1.0),
                ..default()
            },
            ..default()
        },
        Paddle,
        Collider,
        Velocity(Vec3::new(0.0, 0.0, 0.0))
    ));

    // Ball
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Circle {
                radius: 1.0,
            })),
            material: materials.add(Color::WHITE),
            transform: Transform::from_translation(Vec3::new(
                0.0,
                -SCREEN_HEIGHT / 2. + PADDLE_PAD + PADDLE_HEIGHT + BALL_RADIUS/2.,
                0.0
            )).with_scale(Vec2::splat(BALL_RADIUS).extend(1.)),
            ..default()
        },
        Ball,
        Velocity(Vec3::new(0.0, 0.0, 0.0))
    ));

    let brick_cols = ((SCREEN_WIDTH - BRICK_PAD_LR * 2.) / (BRICK_WIDTH + BRICK_SPACE*2.) as f32).floor();
    let brick_rows = ((SCREEN_HEIGHT - BRICK_PAD_TOP - BRICK_PAD_BOTTOM) / (BRICK_HEIGHT + BRICK_SPACE*2.) as f32).floor();

    let left_over = SCREEN_WIDTH - brick_cols * (BRICK_WIDTH + BRICK_SPACE*2.);
    let start_x = -SCREEN_WIDTH / 2. + left_over / 2.;

    // Bricks
    for i in 0..brick_rows as i32 {
        for j in 0..=brick_cols as i32 {
            let x = start_x + j as f32 * (BRICK_WIDTH + BRICK_SPACE);
            let y = -SCREEN_HEIGHT / 2. + BRICK_PAD_BOTTOM + BRICK_HEIGHT / 2. + i as f32 * (BRICK_HEIGHT + BRICK_SPACE*2.);
            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Rectangle::new(1.0, 1.0))),
                    material: materials.add(Color::WHITE),
                    transform: Transform {
                        translation: Vec3::new(
                            x,
                            y,
                            0.0
                        ),
                        scale: Vec3::new(BRICK_WIDTH, BRICK_HEIGHT, 1.0),
                        ..default()
                    },
                    ..default()
                },
                Brick,
                Collider,
            ));
        }
    }
}

fn player_input(
    key: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Velocity, With<Paddle>>,
    mut ball_query: Query<&mut Velocity, (With<Ball>, Without<Paddle>)>,
    turn_state: ResMut<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
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

    if turn_state.get() == &GameState::Start {
        if key.just_pressed(KeyCode::Space) {
            let mut ball_velocity = ball_query.single_mut();
            ball_velocity.0 = Vec3::new(BALL_SPEED, BALL_SPEED, 0.0);
            ball_velocity.0 = ball_velocity.0.normalize() * BALL_SPEED;
            next_state.set(GameState::InGame);
        }
    }
}

fn update_position(time: Res<Time>, mut query: Query<(&Velocity, &mut Transform)>) {
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation += velocity.0 * time.delta_seconds();
    }
}

fn ball_follow(
    mut ball_query: Query<&mut Transform, With<Ball>>,
    paddle_query: Query<&Transform, (With<Paddle>, Without<Ball>)>,
) {
    // This system only runs if the game state is Start

    let mut ball_transform = ball_query.single_mut();
    let paddle_transform = paddle_query.single();

    ball_transform.translation.x = paddle_transform.translation.x;
}

fn check_collisions(
    mut commands: Commands,
    mut ball_query: Query<(&mut Velocity, &mut Transform), (With<Ball>, Without<Paddle>)>,
    mut collider_query: Query<(Entity, &mut Transform, Option<&Brick>, Option<&Paddle>, Option<&mut Velocity>), (With<Collider>, Without<Ball>)>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    let (mut ball_velocity, mut ball_transform) = ball_query.single_mut();
    for (entity, mut transform, maybe_brick, maybe_paddle, maybe_velocity) in collider_query.iter_mut() {
        if maybe_paddle.is_some() {
            let mut velocity = maybe_velocity.unwrap();
            if transform.translation.x - PADDLE_WIDTH/2. < -SCREEN_WIDTH/2. {
                // Clamp left wall
                velocity.0.x = 0.;
                transform.translation.x = -SCREEN_WIDTH/2. + PADDLE_WIDTH/2. + 1.
            }
            if transform.translation.x + PADDLE_WIDTH/2. > SCREEN_WIDTH/2. {
                // Clamp right wall
                velocity.0.x = 0.;
                transform.translation.x = SCREEN_WIDTH/2. - PADDLE_WIDTH/2. - 1.
            }
        }


        let collision = ball_collision(
            BoundingCircle::new(ball_transform.translation.truncate(), BALL_RADIUS),
            Aabb2d::new(
                transform.translation.truncate(),
                transform.scale.truncate() / 2.,
            ),
        );

        if let Some(collision) = collision {
            // Sends a collision event so that other systems can react to the collision
            collision_events.send_default();

            // Bricks should be despawned and increment the scoreboard on collision
            if maybe_brick.is_some() {
                commands.entity(entity).despawn();
                // **score += 1;
            }

            // Reflect the ball's velocity when it collides
            let mut reflect_x = false;
            let mut reflect_y = false;

            // Reflect only if the velocity is in the opposite direction of the collision
            // This prevents the ball from getting stuck inside the bar
            match collision {
                Collision::Left => reflect_x = ball_velocity.0.x > 0.0,
                Collision::Right => reflect_x = ball_velocity.0.x < 0.0,
                Collision::Top => reflect_y = ball_velocity.0.y < 0.0,
                Collision::Bottom => reflect_y = ball_velocity.0.y > 0.0,
            }

            // Reflect velocity on the x-axis if we hit something on the x-axis
            if reflect_x {
                ball_velocity.0.x = -ball_velocity.0.x;
            }

            // Reflect velocity on the y-axis if we hit something on the y-axis
            if reflect_y {
                ball_velocity.0.y = -ball_velocity.0.y;
            }
        }
    }

    if ball_transform.translation.y - BALL_RADIUS < -SCREEN_HEIGHT/2. {
        // Bounce off the top wall
        ball_velocity.0.y = -ball_velocity.0.y;
        ball_transform.translation.y = -SCREEN_HEIGHT/2. + BALL_RADIUS + 1.;
    }
    if ball_transform.translation.y + BALL_RADIUS > SCREEN_HEIGHT/2. {
        // Bounce off the bottom wall
        ball_velocity.0.y = -ball_velocity.0.y;
        ball_transform.translation.y = SCREEN_HEIGHT/2. - BALL_RADIUS - 1.;
    }
    if ball_transform.translation.x - BALL_RADIUS < -SCREEN_WIDTH/2. {
        // Bounce off the left wall
        ball_velocity.0.x = -ball_velocity.0.x;
        ball_transform.translation.x = -SCREEN_WIDTH/2. + BALL_RADIUS + 1.;
    }
    if ball_transform.translation.x + BALL_RADIUS > SCREEN_WIDTH/2. {
        // Bounce off the right wall
        ball_velocity.0.x = -ball_velocity.0.x;
        ball_transform.translation.x = SCREEN_WIDTH/2. - BALL_RADIUS - 1.;
    }
}

fn ball_collision(ball: BoundingCircle, bounding_box: Aabb2d) -> Option<Collision> {
    if !ball.intersects(&bounding_box) {
        return None;
    }

    let closest = bounding_box.closest_point(ball.center());
    let offset = ball.center() - closest;
    let side = if offset.x.abs() > offset.y.abs() {
        if offset.x < 0. {
            Collision::Left
        } else {
            Collision::Right
        }
    } else if offset.y > 0. {
        Collision::Top
    } else {
        Collision::Bottom
    };

    Some(side)
}