use bevy::prelude::*;

#[derive(Component)]
pub struct Ball;

#[derive(Component)]
pub struct PlayerPaddle;

#[derive(Component)]
pub struct OpponentPaddle;

#[derive(Component)]
pub struct Velocity(pub Vec3);

#[derive(Resource)]
pub struct Score {
    pub player: u32,
    pub opponent: u32,
}

impl Default for Score {
    fn default() -> Self {
        Self {
            player: 0,
            opponent: 0,
        }
    }
}

pub enum Scorer {
    Opponent,
    Player
}

#[derive(Event)]
pub struct Scored(pub Scorer);

#[derive(Component)]
pub struct PlayerScoreboard;

#[derive(Component)]
pub struct OpponentScoreboard;