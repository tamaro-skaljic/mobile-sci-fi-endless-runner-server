use spacetimedb::{Identity, SpacetimeType, Timestamp, table};
use spacetimedsl::prelude::*;

use super::level_skin::LevelSkinVariant;
use super::player_movement_trail::PlayerMovementTrailVariant;
use super::player_skin::PlayerSkinVariant;

#[dsl(plural_name = purchases, method(update = false))]
#[table(accessor = purchase, private)]
pub struct Purchase {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(super::player::PlayerId)]
    #[foreign_key(path = super::player, table = player, column = id, on_delete = Delete)]
    player_id: Identity,

    variant: PurchaseVariant,
    gems: u64,
    coins: u64,

    created_at: Timestamp,
}

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum PurchaseVariant {
    Energy,
    Revive,
    Magnet,
    MagnetRangeUpgrade,
    MagnetDurationUpgrade,
    MagnetSpawnChanceUpgrade,
    Shield,
    ShieldCollisionsUpgrade,
    ShieldDurationUpgrade,
    ShieldSpawnChanceUpgrade,
    PlayerSkin(PlayerSkinVariant),
    LevelSkin(LevelSkinVariant),
    PlayerMovementTrail(PlayerMovementTrailVariant),
}

impl std::fmt::Display for PurchaseVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PurchaseVariant::Energy => write!(f, "Energy"),
            PurchaseVariant::Revive => write!(f, "Revive"),
            PurchaseVariant::Magnet => write!(f, "Magnet"),
            PurchaseVariant::MagnetRangeUpgrade => write!(f, "MagnetRangeUpgrade"),
            PurchaseVariant::MagnetDurationUpgrade => write!(f, "MagnetDurationUpgrade"),
            PurchaseVariant::MagnetSpawnChanceUpgrade => write!(f, "MagnetSpawnChanceUpgrade"),
            PurchaseVariant::Shield => write!(f, "Shield"),
            PurchaseVariant::ShieldCollisionsUpgrade => write!(f, "ShieldCollisionsUpgrade"),
            PurchaseVariant::ShieldDurationUpgrade => write!(f, "ShieldDurationUpgrade"),
            PurchaseVariant::ShieldSpawnChanceUpgrade => write!(f, "ShieldSpawnChanceUpgrade"),
            PurchaseVariant::PlayerSkin(_) => write!(f, "PlayerSkin"),
            PurchaseVariant::LevelSkin(_) => write!(f, "LevelSkin"),
            PurchaseVariant::PlayerMovementTrail(_) => write!(f, "PlayerMovementTrail"),
        }
    }
}

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum Currency {
    Gems,
    Coins,
}
