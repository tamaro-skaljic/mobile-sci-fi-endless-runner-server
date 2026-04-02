use spacetimedb::{ReducerContext, reducer};
use spacetimedsl::prelude::*;

use crate::{
    ban_player::return_error_if_player_is_banned,
    player_name_generator::generate_unique_player_name,
    tables::{
        level_skin::LevelSkinVariant,
        player::{
            CreatePlayer, CreatePlayerRow, GetPlayerRowOptionById, PlayerId, UpdatePlayerRowById,
        },
        player_skin::PlayerSkinVariant,
    },
};

#[reducer(client_connected)]
pub fn connect_client(ctx: &ReducerContext) -> Result<(), SpacetimeDSLError> {
    let dsl = dsl(ctx);

    let sender = dsl.ctx().sender();

    match dsl.get_player_by_id(PlayerId::new(sender)) {
        Ok(player) => {
            // this disconnects immediately on error
            return_error_if_player_is_banned(&player)?;

            dsl.update_player_by_id(player)?;
        }
        Err(SpacetimeDSLError::NotFoundError { .. }) => {
            let name = generate_unique_player_name(&dsl);

            dsl.create_player(CreatePlayer {
                id: sender,
                name,
                is_banned: None,
                last_playthrough_id: None,
                next_get_gems_ad_watch_available_at: dsl.ctx().timestamp,
                last_daily_reward_claimed_at: None,
                number_of_claimed_daily_rewards: 0,
                time_difference_from_utc_in_minutes: 0,
                player_skin: PlayerSkinVariant::Default,
                level_skin: LevelSkinVariant::NeonSectorOne,
                player_movement_trail: None,
                level: 1,
            })?;
        }
        Err(e) => return Err(e),
    }

    Ok(())
}
