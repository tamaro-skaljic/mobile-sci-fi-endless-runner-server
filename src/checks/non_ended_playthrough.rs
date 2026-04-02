use spacetimedb::ReducerContext;
use spacetimedsl::prelude::*;

use crate::tables::{
    player::Player,
    playthrough::{GetPlaythroughRowOptionById, Playthrough},
};

pub fn get_non_ended_playthrough(
    dsl: &DSL<'_, ReducerContext>,
    player: &Player,
) -> Option<Playthrough> {
    let playthrough_id = player.get_last_playthrough_id()?;
    let playthrough = dsl.get_playthrough_by_id(playthrough_id).ok()?;
    if playthrough.get_end().is_some() {
        return None;
    }
    Some(playthrough)
}
