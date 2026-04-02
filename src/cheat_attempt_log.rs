use spacetimedb::ReducerContext;
use spacetimedsl::prelude::*;

use crate::tables::cheat_attempt_log::{CreateCheatAttemptLog, CreateCheatAttemptLogRow};
use crate::tables::player::PlayerId;

pub struct CheatAttempt;

pub enum CheatOrError {
    CheatAttempt(CheatAttempt),
    Error(SpacetimeDSLError),
}

impl From<SpacetimeDSLError> for CheatOrError {
    fn from(e: SpacetimeDSLError) -> Self {
        CheatOrError::Error(e)
    }
}

pub fn cheat_attempt(
    dsl: &DSL<'_, ReducerContext>,
    player_id: impl Into<PlayerId>,
    reason: &str,
) -> CheatOrError {
    let player_id = player_id.into();

    match dsl.create_cheat_attempt_log(CreateCheatAttemptLog {
        player_id: player_id.clone(),
        reason: reason.to_string(),
    }) {
        Ok(_) => {}
        Err(err) => return err.into(),
    };

    log::warn!("Cheat attempt by player {player_id:?}: {reason}");

    CheatOrError::CheatAttempt(CheatAttempt)
}

#[macro_export]
macro_rules! or_ok_on_cheat {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err($crate::cheat_attempt_log::CheatOrError::CheatAttempt(_)) => return Ok(()),
            Err($crate::cheat_attempt_log::CheatOrError::Error(e)) => return Err(e),
        }
    };
}
