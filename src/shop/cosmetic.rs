use spacetimedb::ReducerContext;
use spacetimedsl::prelude::*;

use crate::{
    cheat_attempt_log::{CheatOrError, cheat_attempt},
    purchase_price,
    remove_from_wallet::charge_wallet,
    tables::{
        level_skin::{
            GetLevelSkinRowOptionByPlayerIdAndVariant, LevelSkinVariant,
            UpdateLevelSkinRowByPlayerIdAndVariant,
        },
        player::Player,
        player_movement_trail::{
            GetPlayerMovementTrailRowOptionByPlayerIdAndVariant, PlayerMovementTrailVariant,
            UpdatePlayerMovementTrailRowByPlayerIdAndVariant,
        },
        player_skin::{
            GetPlayerSkinRowOptionByPlayerIdAndVariant, PlayerSkinVariant,
            UpdatePlayerSkinRowByPlayerIdAndVariant,
        },
        purchase::Currency,
    },
};

macro_rules! purchase_cosmetic {
    (
        $fn_name:ident,
        $variant_type:ty,
        $get:ident,
        $update:ident,
        $price_fn:path,
        $label:expr
    ) => {
        pub fn $fn_name(
            dsl: &DSL<'_, ReducerContext>,
            player: &Player,
            currency: &Option<Currency>,
            variant: &$variant_type,
        ) -> Result<(u64, u64), CheatOrError> {
            let mut item = dsl.$get(player, variant)?;

            if *item.get_purchased() {
                return Err(cheat_attempt(
                    dsl,
                    player,
                    &format!("Tried to purchase an already purchased {}", $label),
                ));
            }

            let price = $price_fn();
            let charge = charge_wallet(dsl, player, &price, currency)?;

            item.set_purchased(true);
            dsl.$update(item)?;

            Ok(charge)
        }
    };
}

purchase_cosmetic!(
    purchase_player_skin,
    PlayerSkinVariant,
    get_player_skin_by_player_id_and_variant,
    update_player_skin_by_player_id_and_variant,
    purchase_price::player_skin_price,
    "player skin"
);

purchase_cosmetic!(
    purchase_level_skin,
    LevelSkinVariant,
    get_level_skin_by_player_id_and_variant,
    update_level_skin_by_player_id_and_variant,
    purchase_price::level_skin_price,
    "level skin"
);

purchase_cosmetic!(
    purchase_player_movement_trail,
    PlayerMovementTrailVariant,
    get_player_movement_trail_by_player_id_and_variant,
    update_player_movement_trail_by_player_id_and_variant,
    purchase_price::player_movement_trail_price,
    "player movement trail"
);
