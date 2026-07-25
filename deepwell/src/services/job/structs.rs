/*
 * services/job/structs.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

use crate::services::page_revision::RerenderType;
use crate::types::{PageId, RerenderDepth};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case", tag = "job", content = "data")]
pub enum Job {
    RerenderPage {
        id: PageId,
        depth: RerenderDepth,
        r#type: RerenderType,
    },
    PruneSessions,
    PrunePendingUploads,
    PruneText,
    NameChangeRefill,
    LiftExpiredPunishments,
}

impl Job {
    /// Stable, payload-free name for operational logs and metrics.
    pub(crate) const fn kind(&self) -> &'static str {
        match self {
            Self::RerenderPage { .. } => "rerender_page",
            Self::PruneSessions => "prune_sessions",
            Self::PrunePendingUploads => "prune_pending_uploads",
            Self::PruneText => "prune_text",
            Self::NameChangeRefill => "name_change_refill",
            Self::LiftExpiredPunishments => "lift_expired_punishments",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Job;
    use crate::services::page_revision::RerenderType;
    use crate::types::{PageId, RerenderDepth};

    #[test]
    fn job_kind_is_stable_and_omits_payload_values() {
        let rerender = Job::RerenderPage {
            id: PageId {
                site_id: 11,
                category_id: 22,
                page_id: 33,
            },
            depth: RerenderDepth::default(),
            r#type: RerenderType::Full,
        };

        assert_eq!(rerender.kind(), "rerender_page");
        assert_eq!(Job::PruneSessions.kind(), "prune_sessions");
        assert_eq!(Job::PrunePendingUploads.kind(), "prune_pending_uploads");
        assert_eq!(Job::PruneText.kind(), "prune_text");
        assert_eq!(Job::NameChangeRefill.kind(), "name_change_refill");
        assert_eq!(
            Job::LiftExpiredPunishments.kind(),
            "lift_expired_punishments"
        );
        assert!(!rerender.kind().contains("11"));
        assert!(!rerender.kind().contains("22"));
        assert!(!rerender.kind().contains("33"));
    }
}
