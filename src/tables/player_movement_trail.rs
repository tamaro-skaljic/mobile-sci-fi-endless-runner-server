use spacetimedb::{Identity, SpacetimeType, Timestamp, table};
use spacetimedsl::prelude::*;
use strum::{Display, EnumIter};

#[dsl(
    plural_name = player_movement_trails,
    method(update = true, delete = true),
    unique_index(name = player_id_and_variant),
)]
#[table(
    accessor = player_movement_trail,
    index(accessor = player_id_and_variant, btree(columns = [player_id, variant])),
    private,
)]
pub struct PlayerMovementTrail {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    data_id: u64,

    #[index(btree)]
    #[use_wrapper(super::player::PlayerId)]
    #[foreign_key(path = super::player, table = player, column = id, on_delete = Delete)]
    pub(crate) player_id: Identity,

    #[index(btree)]
    variant: PlayerMovementTrailVariant,

    pub purchased: bool,

    modified_at: Option<Timestamp>,
}

#[derive(SpacetimeType, Copy, Clone, Debug, PartialEq, Display, EnumIter)]
pub enum PlayerMovementTrailVariant {
    Cosmos,
    Dark,
    Electric,
    Fire,
    Ice,
    Nature,
    Void,
    Water,
}
