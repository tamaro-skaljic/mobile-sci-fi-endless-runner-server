use spacetimedb::{ReducerContext, reducer};
use spacetimedsl::prelude::*;

use crate::{
    authenticated_player::get_authenticated_player,
    constants::{MAX_PLAYER_NAME_LENGTH, MIN_PLAYER_NAME_LENGTH},
    tables::{
        player::UpdatePlayerRowById,
        player_name::{
            CreatePlayerName, CreatePlayerNameRow, GetPlayerNameRowOptionByName, ModerationStatus,
        },
    },
};

#[reducer]
pub fn rename_player(ctx: &ReducerContext, name: String) -> Result<(), SpacetimeDSLError> {
    let (dsl, mut player) = get_authenticated_player(ctx)?;

    if name.len() < MIN_PLAYER_NAME_LENGTH || name.len() > MAX_PLAYER_NAME_LENGTH {
        return Err(SpacetimeDSLError::Error(format!(
            "Player name must be between {MIN_PLAYER_NAME_LENGTH} and {MAX_PLAYER_NAME_LENGTH} characters"
        )));
    }

    match dsl.get_player_name_by_name(&name) {
        Ok(moderation) => match moderation.get_status() {
            ModerationStatus::Approved | ModerationStatus::Pending => {
                player.set_name(name);
                dsl.update_player_by_id(player)?;
            }
            ModerationStatus::Rejected => {
                return Err(SpacetimeDSLError::Error(
                    "Player name rejected.".to_string(),
                ));
            }
        },
        Err(_) => {
            dsl.create_player_name(CreatePlayerName {
                name: name.clone(),
                status: ModerationStatus::Pending,
            })?;

            player.set_name(name);
            dsl.update_player_by_id(player)?;
        }
    }

    Ok(())
}
