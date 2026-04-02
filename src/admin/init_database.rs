use spacetimedb::{ReducerContext, ScheduleAt, TimeDuration, reducer};
use spacetimedsl::prelude::*;

use crate::scheduled_functions::check_player_names::{
    CreateCheckPlayerNamesJob, CreateCheckPlayerNamesJobRow,
};
use crate::scheduled_functions::refresh_google_play_android_developer_api_access_token::schedule_google_play_android_developer_api_access_token_refresh;

#[reducer(init)]
pub fn init_database(ctx: &ReducerContext) -> Result<(), SpacetimeDSLError> {
    let dsl = dsl(ctx);

    // schedule the first player name moderation job to run in one hour
    dsl.create_check_player_names_job(CreateCheckPlayerNamesJob {
        scheduled_at: ScheduleAt::Interval(TimeDuration::from_micros(3_600 * 1_000_000)),
    })?;

    // schedule the first Google Play access token refresh to run immediately
    schedule_google_play_android_developer_api_access_token_refresh(&dsl)?;

    Ok(())
}
