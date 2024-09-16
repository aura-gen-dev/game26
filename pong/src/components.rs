use bevy::prelude::*;

#[derive(Component)]
pub struct Ball;

#[derive(Component)]
pub struct PlayerPaddle;

#[derive(Component)]
pub struct OpponentPaddle;

#[derive(Component)]
pub struct Velocity(pub Vec3);
