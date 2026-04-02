use spacetimedb::{Timestamp, table};
use spacetimedsl::prelude::*;

#[dsl(plural_name = configs, method(update = true, delete = true))]
#[table(accessor = config, private)]
pub struct Config {
    #[primary_key]
    #[create_wrapper]
    key: String,

    pub value: String,

    created_at: Timestamp,

    modified_at: Option<Timestamp>,
}
