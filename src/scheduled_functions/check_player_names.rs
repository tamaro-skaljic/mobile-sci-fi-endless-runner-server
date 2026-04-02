use spacetimedb::{ProcedureContext, ScheduleAt, procedure, table};
use spacetimedsl::prelude::*;

#[dsl(plural_name = check_player_names_jobs, method(update = false, delete = true))]
#[table(accessor = check_player_names_job, scheduled(check_player_names), private)]
pub struct CheckPlayerNamesJob {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    scheduled_id: u64,

    scheduled_at: ScheduleAt,
}

/// Checks all pending player name moderation results for compliance and updates their status accordingly
#[procedure]
pub fn check_player_names(
    _ctx: &mut ProcedureContext,
    _job: CheckPlayerNamesJob,
) -> Result<(), String> {
    // FIXME: Implement actual moderation logic.

    // let dsl = dsl(ctx);

    // let unchecked_names = dsl
    //     .get_player_names_by_status(&ModerationStatus::Pending)
    //     .collect_vec();
    //
    // for unchecked_name in unchecked_names {
    //     if todo!("Implement actual moderation logic") {
    //         unchecked_name.set_status(ModerationStatus::Approved);
    //     } else {
    //         unchecked_name.set_status(ModerationStatus::Rejected);
    //     }
    //     dsl.update_player_name_by_name(unchecked_name)?;
    // }

    Ok(())
}
