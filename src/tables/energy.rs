use spacetimedb::{Identity, SpacetimeType, Timestamp, table};
use spacetimedsl::prelude::*;

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum EnergyChange {
    Increasing(Timestamp),
    Decreasing(Timestamp),
}

impl EnergyChange {
    pub fn timestamp(&self) -> Timestamp {
        match self {
            EnergyChange::Increasing(timestamp) | EnergyChange::Decreasing(timestamp) => *timestamp,
        }
    }
}

#[dsl(plural_name = energy, method(update = true, delete = true))]
#[table(accessor = energy, private)]
pub struct Energy {
    #[primary_key]
    #[use_wrapper(super::player::PlayerId)]
    #[foreign_key(path = super::player, table = player, column = id, on_delete = Delete)]
    pub(crate) player_id: Identity,

    pub energy_boundary_reached_at: Option<EnergyChange>,

    pub last_energy_regeneration_at: Timestamp,

    pub last_energy_calculation_at: Timestamp,

    pub energy: u8,

    created_at: Timestamp,
}
