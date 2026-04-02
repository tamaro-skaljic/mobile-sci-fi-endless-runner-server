use spacetimedb::{Identity, SpacetimeType, Timestamp, table};
use spacetimedsl::prelude::*;

#[dsl(plural_name = revives, method(update = true, delete = true))]
#[table(accessor = revive, private)]
pub struct Revive {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(super::player::PlayerId)]
    #[foreign_key(path = super::player, table = player, column = id, on_delete = Delete)]
    player_id: Identity,

    #[index(btree)]
    #[use_wrapper(super::playthrough::PlaythroughId)]
    #[foreign_key(path = super::playthrough, table = playthrough, column = id, on_delete = Delete)]
    playthrough_id: u64,

    revive_type: ReviveType,

    created_at: Timestamp,

    modified_at: Option<Timestamp>,
}

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum ReviveType {
    /// How many gems the player paid for the revive.
    Gems(u8),
    AdWatch,
}
