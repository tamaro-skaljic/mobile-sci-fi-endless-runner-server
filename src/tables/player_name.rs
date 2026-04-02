use spacetimedb::{SpacetimeType, Timestamp, table};
use spacetimedsl::prelude::*;

#[dsl(plural_name = player_names, method(update = true, delete = false))]
#[table(accessor = player_name, private)]
pub struct PlayerName {
    #[primary_key]
    #[create_wrapper(PlayerNameKey)]
    name: String,

    #[index(btree)]
    pub status: ModerationStatus,

    created_at: Timestamp,

    modified_at: Option<Timestamp>,
}

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum ModerationStatus {
    Pending,
    Approved,
    Rejected,
}
