use crate::printer::PrintableAuditEvent;
use openapi::models::QueryAuditEventsResponse;

pub struct AuditEventsIterator {
    response: QueryAuditEventsResponse,
    index: usize,
}

impl AuditEventsIterator {
    pub fn new(response: QueryAuditEventsResponse) -> Self {
        Self { response, index: 0 }
    }
}

impl Iterator for AuditEventsIterator {
    type Item = PrintableAuditEvent;

    fn next(&mut self) -> Option<Self::Item> {
        let event = self.response.audit_events.get(self.index)?;

        // Find actor email
        let actor_email = self
            .response
            .users
            .as_ref()
            .and_then(|users| users.iter().find(|user| user.id == event.actor_user_id))
            .map(|user| user.email.clone())
            .unwrap_or_else(|| format!("Unknown User ({})", event.actor_user_id));

        // Find actor tenant name
        let actor_tenant_name = self
            .response
            .tenants
            .iter()
            .find(|tenant| tenant.id == event.actor_tenant_id)
            .map(|tenant| tenant.name.clone())
            .unwrap_or_else(|| format!("Unknown Tenant ({})", event.actor_tenant_id));

        // Resolve dataset names
        let dataset_names = event
            .dataset_ids
            .as_ref()
            .map(|dataset_ids| {
                dataset_ids
                    .iter()
                    .map(|dataset_id| {
                        self.response
                            .datasets
                            .as_ref()
                            .and_then(|datasets| {
                                datasets.iter().find(|dataset| dataset.id == *dataset_id)
                            })
                            .map(|dataset| dataset.name.clone())
                            .unwrap_or_else(|| format!("Unknown Dataset ({})", dataset_id))
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Resolve project names
        let project_names = event
            .project_ids
            .as_ref()
            .map(|project_ids| {
                project_ids
                    .iter()
                    .map(|project_id| {
                        self.response
                            .projects
                            .as_ref()
                            .and_then(|projects| {
                                projects.iter().find(|project| project.id == *project_id)
                            })
                            .map(|project| project.name.clone())
                            .unwrap_or_else(|| format!("Unknown Project ({})", project_id))
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Resolve tenant names
        let tenant_names = event
            .tenant_ids
            .iter()
            .map(|tenant_id| {
                self.response
                    .tenants
                    .iter()
                    .find(|tenant| tenant.id == *tenant_id)
                    .map(|tenant| tenant.name.clone())
                    .unwrap_or_else(|| format!("Unknown Tenant ({})", tenant_id))
            })
            .collect();

        let printable_event = PrintableAuditEvent {
            event: event.clone(),
            dataset_names,
            project_names,
            tenant_names,
            actor_email,
            actor_tenant_name,
        };

        self.index += 1;
        Some(printable_event)
    }
}

// Extension trait to add the clean interface to QueryAuditEventsResponse
pub trait AuditEventsResponseExt {
    fn into_iter_printable(self) -> AuditEventsIterator;
}

impl AuditEventsResponseExt for QueryAuditEventsResponse {
    fn into_iter_printable(self) -> AuditEventsIterator {
        AuditEventsIterator::new(self)
    }
}
