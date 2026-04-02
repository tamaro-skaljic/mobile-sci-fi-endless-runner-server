use spacetimedb::{SpacetimeType, ViewContext};
use strum::IntoEnumIterator;

use crate::{
    purchase_price::{
        PricingMode, level_skin_price, player_movement_trail_price, player_skin_price,
    },
    tables::{
        level_skin::{LevelSkinVariant, level_skin__view},
        player::player__view,
        player_movement_trail::{PlayerMovementTrailVariant, player_movement_trail__view},
        player_skin::{PlayerSkinVariant, player_skin__view},
    },
};

macro_rules! cosmetic_widget_view {
    (
        $view_name:ident, $widget_enum:ident, $widget_entry:ident,
        $variant_type:ty, $table_accessor:ident, $price_fn:path,
        direct_applied: $get_applied:ident
    ) => {
        cosmetic_widget_view!(@types $widget_enum, $widget_entry, $variant_type);

        #[spacetimedb::view(accessor = $view_name, public)]
        pub fn $view_name(ctx: &ViewContext) -> Vec<$widget_entry> {
            let Some(player) = ctx.db.player().id().find(ctx.sender()) else {
                return Vec::new();
            };

            let owned: Vec<_> = ctx
                .db
                .$table_accessor()
                .player_id()
                .filter(&ctx.sender())
                .collect();

            <$variant_type>::iter()
                .map(|variant| {
                    let purchased = owned
                        .iter()
                        .any(|row| *row.get_variant() == variant && *row.get_purchased());
                    let is_applied = variant == *player.$get_applied();
                    let status = $widget_enum::from_state(purchased, is_applied, $price_fn());
                    $widget_entry { variant, status }
                })
                .collect()
        }
    };

    (
        $view_name:ident, $widget_enum:ident, $widget_entry:ident,
        $variant_type:ty, $table_accessor:ident, $price_fn:path,
        optional_applied: $get_applied:ident
    ) => {
        cosmetic_widget_view!(@types $widget_enum, $widget_entry, $variant_type);

        #[spacetimedb::view(accessor = $view_name, public)]
        pub fn $view_name(ctx: &ViewContext) -> Vec<$widget_entry> {
            let Some(player) = ctx.db.player().id().find(ctx.sender()) else {
                return Vec::new();
            };

            let owned: Vec<_> = ctx
                .db
                .$table_accessor()
                .player_id()
                .filter(&ctx.sender())
                .collect();

            <$variant_type>::iter()
                .map(|variant| {
                    let purchased = owned
                        .iter()
                        .any(|row| *row.get_variant() == variant && *row.get_purchased());
                    let is_applied = player.$get_applied().as_ref() == Some(&variant);
                    let status = $widget_enum::from_state(purchased, is_applied, $price_fn());
                    $widget_entry { variant, status }
                })
                .collect()
        }
    };

    (@types $widget_enum:ident, $widget_entry:ident, $variant_type:ty) => {
        #[derive(SpacetimeType, Clone, Debug, PartialEq)]
        pub enum $widget_enum {
            Purchased,
            Applied,
            Purchaseable(PricingMode),
        }

        impl $widget_enum {
            fn from_state(purchased: bool, is_applied: bool, price: PricingMode) -> Self {
                if is_applied {
                    Self::Applied
                } else if purchased {
                    Self::Purchased
                } else {
                    Self::Purchaseable(price)
                }
            }
        }

        #[derive(SpacetimeType, Clone, Debug, PartialEq)]
        pub struct $widget_entry {
            pub variant: $variant_type,
            pub status: $widget_enum,
        }
    };
}

cosmetic_widget_view!(
    player_skin_widgets,
    PlayerSkinWidget,
    PlayerSkinWidgetEntry,
    PlayerSkinVariant,
    player_skin,
    player_skin_price,
    direct_applied: get_player_skin
);

cosmetic_widget_view!(
    level_skin_widgets,
    LevelSkinWidget,
    LevelSkinWidgetEntry,
    LevelSkinVariant,
    level_skin,
    level_skin_price,
    direct_applied: get_level_skin
);

cosmetic_widget_view!(
    player_movement_trail_widgets,
    PlayerMovementTrailWidget,
    PlayerMovementTrailWidgetEntry,
    PlayerMovementTrailVariant,
    player_movement_trail,
    player_movement_trail_price,
    optional_applied: get_player_movement_trail
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applied_when_purchased_and_active() {
        let status = PlayerSkinWidget::from_state(true, true, PricingMode::Gems(250));
        assert_eq!(status, PlayerSkinWidget::Applied);
    }

    #[test]
    fn purchased_when_purchased_but_not_active() {
        let status = PlayerSkinWidget::from_state(true, false, PricingMode::Gems(250));
        assert_eq!(status, PlayerSkinWidget::Purchased);
    }

    #[test]
    fn purchaseable_when_not_purchased() {
        let status = PlayerSkinWidget::from_state(false, false, PricingMode::Gems(250));
        assert_eq!(
            status,
            PlayerSkinWidget::Purchaseable(PricingMode::Gems(250))
        );
    }

    #[test]
    fn applied_takes_precedence_even_if_not_marked_purchased() {
        let status = PlayerSkinWidget::from_state(false, true, PricingMode::Gems(250));
        assert_eq!(status, PlayerSkinWidget::Applied);
    }
}
