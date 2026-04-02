use spacetimedb::{ReducerContext, SpacetimeType, reducer};
use spacetimedsl::prelude::*;

use crate::{
    checks::is_admin_client::verify_admin_access,
    or_ok_on_cheat,
    tables::config::{
        ConfigKey, CreateConfig, CreateConfigRow, DeleteConfigRowByKey, GetConfigRowOptionByKey,
        UpdateConfigRowByKey,
    },
};

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum ConfigAction {
    Create(ConfigKeyValue),
    Update(ConfigKeyValue),
    Delete(ConfigKey),
}

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub struct ConfigKeyValue {
    pub key: ConfigKey,
    pub value: String,
}

#[reducer]
pub fn manage_config(ctx: &ReducerContext, action: ConfigAction) -> Result<(), SpacetimeDSLError> {
    let dsl = dsl(ctx);

    or_ok_on_cheat!(verify_admin_access(&dsl, "manage config"));

    match action {
        ConfigAction::Create(key_value) => {
            dsl.create_config(CreateConfig {
                key: key_value.key.value(),
                value: key_value.value,
            })?;
        }
        ConfigAction::Update(key_value) => {
            let mut config = dsl.get_config_by_key(&key_value.key.value())?;
            config.set_value(key_value.value);
            dsl.update_config_by_key(config)?;
        }
        ConfigAction::Delete(key) => {
            dsl.delete_config_by_key(&key.value())?;
        }
    }

    Ok(())
}
