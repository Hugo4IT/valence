use bevy_ecs::prelude::*;

use crate::protocol::{Decode, Encode};

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, Encode, Decode, Component)]
pub enum GameMode {
    #[default]
    Survival,
    Creative,
    Adventure,
    Spectator,
}

impl GameMode {
    /// Converts gamemode to its number representation
    /// ### Values
    /// 0. Survival
    /// 1. Creative
    /// 2. Adventure
    /// 3. Spectator
    pub fn to_index(self) -> usize {
        match self {
            Self::Survival => 0,
            Self::Creative => 1,
            Self::Adventure => 2,
            Self::Spectator => 3,
        }
    }
}
