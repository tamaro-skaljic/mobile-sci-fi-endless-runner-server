use std::fmt;

use spacetimedb::{ReducerContext, SpacetimeType, reducer};
use spacetimedsl::prelude::*;

use crate::{
    cheat_attempt_log::{CheatOrError, cheat_attempt},
    or_ok_on_cheat,
    tables::{
        level_skin::{GetLevelSkinRowOptionByPlayerIdAndVariant, LevelSkinVariant},
        player::{GetPlayerRowOptionById, Player, PlayerId, UpdatePlayerRowById},
        player_movement_trail::{
            GetPlayerMovementTrailRowOptionByPlayerIdAndVariant, PlayerMovementTrailVariant,
        },
        player_skin::{GetPlayerSkinRowOptionByPlayerIdAndVariant, PlayerSkinVariant},
    },
};

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum CosmeticVariant {
    PlayerSkin(PlayerSkinVariant),
    LevelSkin(LevelSkinVariant),
    PlayerMovementTrail(PlayerMovementTrailVariant),
}

impl fmt::Display for CosmeticVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CosmeticVariant::PlayerSkin(v) => write!(f, "player skin {v}"),
            CosmeticVariant::LevelSkin(v) => write!(f, "level skin {v}"),
            CosmeticVariant::PlayerMovementTrail(v) => write!(f, "player movement trail {v}"),
        }
    }
}

fn check_cosmetic(
    dsl: &DSL<'_, ReducerContext>,
    player: &Player,
    is_purchased: bool,
    is_already_applied: bool,
    variant: CosmeticVariant,
) -> Result<(), CheatOrError> {
    if !is_purchased {
        return Err(cheat_attempt(
            dsl,
            player,
            &format!("Tried to apply {variant} which is not purchased"),
        ));
    }

    if is_already_applied {
        return Err(cheat_attempt(
            dsl,
            player,
            &format!("Tried to apply {variant} which is already applied"),
        ));
    }

    Ok(())
}

#[reducer]
pub fn apply_cosmetic(
    ctx: &ReducerContext,
    cosmetic: CosmeticVariant,
) -> Result<(), SpacetimeDSLError> {
    let dsl = dsl(ctx);

    let sender = dsl.ctx().sender();
    let mut player = dsl.get_player_by_id(PlayerId::new(sender))?;

    match cosmetic {
        CosmeticVariant::PlayerSkin(variant) => {
            let skin = dsl.get_player_skin_by_player_id_and_variant(player.get_id(), &variant)?;

            or_ok_on_cheat!(check_cosmetic(
                &dsl,
                &player,
                skin.purchased,
                *player.get_player_skin() == variant,
                CosmeticVariant::PlayerSkin(variant),
            ));

            player.set_player_skin(variant);
        }
        CosmeticVariant::LevelSkin(variant) => {
            let skin = dsl.get_level_skin_by_player_id_and_variant(player.get_id(), &variant)?;

            or_ok_on_cheat!(check_cosmetic(
                &dsl,
                &player,
                skin.purchased,
                *player.get_level_skin() == variant,
                CosmeticVariant::LevelSkin(variant),
            ));

            player.set_level_skin(variant);
        }
        CosmeticVariant::PlayerMovementTrail(variant) => {
            let trail =
                dsl.get_player_movement_trail_by_player_id_and_variant(player.get_id(), &variant)?;

            or_ok_on_cheat!(check_cosmetic(
                &dsl,
                &player,
                trail.purchased,
                *player.get_player_movement_trail() == Some(variant),
                CosmeticVariant::PlayerMovementTrail(variant),
            ));

            player.set_player_movement_trail(Some(variant));
        }
    }

    dsl.update_player_by_id(player)?;

    Ok(())
}
