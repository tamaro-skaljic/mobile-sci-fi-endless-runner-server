use spacetimedb::{Identity, SpacetimeType, Timestamp, table};
use spacetimedsl::prelude::*;
use strum::{Display, EnumIter};

#[dsl(
    plural_name = player_skins,
    method(update = true, delete = true),
    unique_index(name = player_id_and_variant),
)]
#[table(
    accessor = player_skin,
    index(accessor = player_id_and_variant, btree(columns = [player_id, variant])),
    private,
)]
pub struct PlayerSkin {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    data_id: u64,

    #[index(btree)]
    #[use_wrapper(super::player::PlayerId)]
    #[foreign_key(path = super::player, table = player, column = id, on_delete = Delete)]
    pub(crate) player_id: Identity,

    #[index(btree)]
    variant: PlayerSkinVariant,

    pub purchased: bool,

    modified_at: Option<Timestamp>,
}

#[derive(SpacetimeType, Copy, Clone, Debug, PartialEq, Display, EnumIter)]
pub enum PlayerSkinVariant {
    Default,
    TitanCore,
    CrimsonReactor,
    NeonHelix,
    QuantumBloom,
    EmeraldCircuit,
    SolarOverdrive,
    InfernoTorque,
    RadialSurge,
    AuroraPulse,
    VerdantRift,
    MoltenFang,
    ObsidianMaw,
    VoidShatter,
    PlasmaCrown,
    RedDust,
    Greeno,
}
