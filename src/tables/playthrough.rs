use spacetimedb::{Identity, SpacetimeType, Timestamp, table};
use spacetimedsl::prelude::*;

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum PauseReason {
    Pause,
    Revive,
    OutOfEnergy,
}

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub struct PauseEntry {
    pub begin: Timestamp,
    pub end: Option<Timestamp>,
    pub reason: PauseReason,
}

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum EndReason {
    UserRequest,
    GameClose,
    Death,
    NoRevive,
    OutOfEnergy,
}

#[dsl(plural_name = playthroughs, method(update = true, delete = true))]
#[table(accessor = playthrough, private)]
pub struct Playthrough {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = super::revive, table = revive)]
    pub(crate) id: u64,

    #[index(btree)]
    #[use_wrapper(super::player::PlayerId)]
    #[foreign_key(path = super::player, table = player, column = id, on_delete = Delete)]
    player_id: Identity,

    pub score: u64,

    pub coins: u64,

    pub gems: u64,

    pub is_high_score: bool,

    pub revive_count: u8,

    pub coins_doubled: bool,

    pub energy_purchases: u8,

    pub levels: u32,

    pub pauses: Vec<PauseEntry>,

    pub end_reason: Option<EndReason>,

    pub begin: Timestamp,

    pub end: Option<Timestamp>,

    created_at: Timestamp,

    modified_at: Option<Timestamp>,
}
