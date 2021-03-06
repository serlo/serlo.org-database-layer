use std::convert::TryFrom;

use serde::Serialize;

use super::abstract_event::AbstractEvent;
use super::EventError;

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevisionEvent {
    repository_id: i32,
    revision_id: i32,
    reason: String,
}

impl TryFrom<&AbstractEvent> for RevisionEvent {
    type Error = EventError;

    fn try_from(abstract_event: &AbstractEvent) -> Result<Self, Self::Error> {
        let repository_id = abstract_event.uuid_parameters.try_get("repository")?;
        let revision_id = abstract_event.object_id;
        let reason = abstract_event.string_parameters.get_or("reason", "");

        Ok(RevisionEvent {
            repository_id,
            revision_id,
            reason,
        })
    }
}
