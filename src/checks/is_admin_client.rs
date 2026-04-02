use spacetimedb::ReducerContext;
use spacetimedsl::prelude::*;

use crate::{
    cheat_attempt_log::{CheatOrError, cheat_attempt},
    tables::player::PlayerId,
};

pub fn verify_admin_access(
    dsl: &DSL<'_, ReducerContext>,
    action: &str,
) -> Result<(), CheatOrError> {
    if dsl.ctx().sender() != dsl.ctx().module_identity()? {
        return Err(cheat_attempt(
            dsl,
            PlayerId::new(dsl.ctx().sender()),
            &format!("Tried to {action}, which only administrators can"),
        ));
    }

    Ok(())
}
