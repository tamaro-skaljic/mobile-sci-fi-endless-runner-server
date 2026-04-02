use spacetimedb::ReducerContext;
use spacetimedb::rand::Rng;
use spacetimedsl::prelude::*;

use crate::tables::player::GetPlayerRowOptionByName;

/// Generates a unique random player name in the format "Player" + 10-digit number.
pub fn generate_unique_player_name(dsl: &DSL<'_, ReducerContext>) -> String {
    loop {
        let player_name = generate_player_name(dsl);

        if player_name_is_unique(dsl, &player_name) {
            return player_name;
        }
    }
}

/// Generates a player name in the format "Player" + 10-digit number.
fn generate_player_name(dsl: &DSL<'_, ReducerContext>) -> String {
    let number = dsl
        .ctx()
        .rng()
        .gen_range(1_000_000_000u64..=9_999_999_999u64);

    format!("Player{number}")
}

fn player_name_is_unique(dsl: &DSL<'_, ReducerContext>, name: &str) -> bool {
    // If no player with the generated name exists, it's unique
    dsl.get_player_by_name(name).is_err()
}
