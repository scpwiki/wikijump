/*
 * tests/job_queue.rs
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

use deepwell::api::{
    ServerState, build_server_at,
    build_server_state_without_workers_with_job_queue_namespace,
};
use deepwell::config::{Config, Secrets};
use deepwell::constants::{ADMIN_USER_ID, SYSTEM_USER_ID};
use deepwell::license::License;
use deepwell::models::audit_log::{self, Entity as AuditLog};
use deepwell::models::known_user::Entity as KnownUserTable;
use deepwell::models::page::{self, Entity as PageTable, Model as PageModel};
use deepwell::models::page_category::{self, Entity as PageCategory};
use deepwell::models::page_connection::{self, Entity as PageConnection};
use deepwell::models::relation::{self, Entity as RelationTable};
use deepwell::models::role::{self, Entity as RoleTable};
use deepwell::models::role_permission::{self, Entity as RolePermissionTable};
use deepwell::models::session::Entity as SessionTable;
use deepwell::models::site::{Entity as SiteTable, Model as SiteModel};
use deepwell::models::user::Entity as UserTable;
use deepwell::redis::connect_with_namespace;
use deepwell::services::category::CategoryService;
use deepwell::services::job::{
    JOB_QUEUE_DELAY, JOB_QUEUE_MAXIMUM_SIZE, JOB_QUEUE_NAME, JOB_QUEUE_PROCESS_TIME, Job,
    JobService,
};
use deepwell::services::page_revision::{PageRevisionService, RerenderType};
use deepwell::services::permission::PermissionService;
use deepwell::services::public_cache::PublicContentCache;
use deepwell::services::role::{
    InternalCreateRoleInput, RoleService, SystemRole, UpdateRolePermissionsInput,
};
use deepwell::services::session::{CreateSession, SessionService};
use deepwell::services::site::{CreateSite, SiteService};
use deepwell::services::{OutdateService, ServiceContext};
use deepwell::types::{
    Action, ConnectionType, PageId, Permission, Reference, RelationObjectType,
    RerenderDepth, Resource,
};
use redis::AsyncCommands;
use rsmq_async::{Rsmq, RsmqConnection};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, Set,
    TransactionTrait,
};
use serde_json::{Value, json};
use std::collections::HashSet;
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::panic::AssertUnwindSafe;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use time::OffsetDateTime;
use uuid::Uuid;

use futures::FutureExt;

async fn cleanup_namespace(redis_url: &str, namespace: &str) {
    let client =
        redis::Client::open(redis_url).expect("failed to construct Redis client");
    let mut connection = client
        .get_multiplexed_async_connection()
        .await
        .expect("failed to connect to Redis for cleanup");
    let keys: Vec<String> = redis::cmd("KEYS")
        .arg(format!("{namespace}:*"))
        .query_async(&mut connection)
        .await
        .expect("failed to list test Redis keys");
    if !keys.is_empty() {
        let _: usize = connection
            .del(keys)
            .await
            .expect("failed to delete test Redis keys");
    }
}

#[tokio::test]
async fn existing_job_queue_receives_one_expired_punishment_seed() {
    let redis_url =
        env::var("REDIS_URL").expect("REDIS_URL must be set for integration tests");
    let run_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock predates Unix epoch")
        .as_nanos();
    let namespace = format!("rsmq-test-{}-{run_id}", std::process::id());

    let client = redis::Client::open(redis_url.as_str())
        .expect("failed to construct test Redis client");
    let connection = client
        .get_multiplexed_async_connection()
        .await
        .expect("failed to connect to test Redis");
    let mut rsmq = Rsmq::new_with_connection(connection, false, Some(namespace.as_str()))
        .await
        .expect("failed to construct test RSMQ client");
    rsmq.create_queue(
        JOB_QUEUE_NAME,
        JOB_QUEUE_PROCESS_TIME,
        JOB_QUEUE_DELAY,
        JOB_QUEUE_MAXIMUM_SIZE,
    )
    .await
    .expect("failed to create pre-existing test queue");
    for job in [Job::PruneSessions, Job::PrunePendingUploads, Job::PruneText] {
        JobService::queue_job_inner(&mut rsmq, &job, None)
            .await
            .expect("failed to seed pre-existing recurring job");
    }

    let (_, mut first_start) = connect_with_namespace(&redis_url, &namespace)
        .await
        .expect("first startup against existing queue failed");
    assert_eq!(
        first_start
            .get_queue_attributes(JOB_QUEUE_NAME)
            .await
            .expect("failed to inspect first startup queue")
            .msgs,
        4,
    );

    let (_, mut second_start) = connect_with_namespace(&redis_url, &namespace)
        .await
        .expect("second startup against existing queue failed");
    assert_eq!(
        second_start
            .get_queue_attributes(JOB_QUEUE_NAME)
            .await
            .expect("failed to inspect second startup queue")
            .msgs,
        4,
        "restart added a duplicate recurring job",
    );

    second_start
        .delete_queue(JOB_QUEUE_NAME)
        .await
        .expect("failed to remove the queue-recreation fixture");

    let (_, mut recreated) = connect_with_namespace(&redis_url, &namespace)
        .await
        .expect("startup after queue recreation failed");
    assert_eq!(
        recreated
            .get_queue_attributes(JOB_QUEUE_NAME)
            .await
            .expect("failed to inspect recreated queue")
            .msgs,
        4,
        "a stale seed marker suppressed jobs in a recreated queue",
    );

    let mut jobs = Vec::new();
    while let Some(message) = recreated
        .receive_message::<Vec<u8>>(JOB_QUEUE_NAME, None)
        .await
        .expect("failed to receive queued job")
    {
        jobs.push(
            serde_json::from_slice::<Job>(&message.message)
                .expect("queued job payload was malformed"),
        );
        recreated
            .delete_message(JOB_QUEUE_NAME, &message.id)
            .await
            .expect("failed to delete received test job");
    }

    assert_eq!(
        jobs.iter()
            .filter(|job| matches!(job, Job::LiftExpiredPunishments))
            .count(),
        1,
    );
    assert_eq!(jobs.len(), 4);

    cleanup_namespace(&redis_url, &namespace).await;
}

async fn clear_job_queue(rsmq: &mut Rsmq) {
    while let Some(message) = rsmq
        .receive_message::<Vec<u8>>(JOB_QUEUE_NAME, None)
        .await
        .expect("failed to receive queued setup job")
    {
        rsmq.delete_message(JOB_QUEUE_NAME, &message.id)
            .await
            .expect("failed to delete queued setup job");
    }
}

async fn queued_job_count(rsmq: &mut Rsmq) -> u64 {
    rsmq.get_queue_attributes(JOB_QUEUE_NAME)
        .await
        .expect("failed to inspect isolated job queue")
        .msgs
}

async fn create_admin_session(state: &ServerState) -> String {
    let transaction = state
        .database
        .begin()
        .await
        .expect("admin session transaction should begin");
    let session_token = {
        let ctx = ServiceContext::new(state, &transaction);
        SessionService::create(
            &ctx,
            CreateSession {
                user_id: ADMIN_USER_ID,
                ip_address: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 61)),
                user_agent: "issue 1061 post-commit worker test".to_owned(),
                restricted: false,
            },
        )
        .await
        .expect("admin test session should be created")
    };
    transaction
        .commit()
        .await
        .expect("admin session transaction should commit");
    session_token
}

async fn create_editable_site_fixture(state: &ServerState, site_slug: &str) -> i64 {
    let transaction = state
        .database
        .begin()
        .await
        .expect("editable site fixture transaction should begin");
    let site_id = {
        let ctx = ServiceContext::new(state, &transaction);
        let site = SiteService::create(
            &ctx,
            CreateSite {
                slug: site_slug.to_owned(),
                name: String::from("Post-commit rerender fixture"),
                tagline: String::new(),
                description: String::from("Isolated editable job queue test site"),
                default_page: None,
                layout: Some(ftml::layout::Layout::Wikidot),
                license: License::CcBySa40,
                locale: String::from("en"),
                ip_address: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 61)),
            },
            None,
        )
        .await
        .expect("editable site fixture should be created");
        for category in ["_default", "component"] {
            CategoryService::get_or_create(&ctx, site.site_id, category)
                .await
                .expect("editable page category fixture should be created");
        }
        let everyone = RoleService::create(
            &ctx,
            InternalCreateRoleInput {
                site_id: site.site_id,
                name: SystemRole::Everyone.to_string(),
                description: Some(String::from("Editable page fixture role")),
                is_virtual: true,
                parent_role_id: None,
                creating_user_id: SYSTEM_USER_ID,
                ip_address: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 61)),
            },
        )
        .await
        .expect("editable page role fixture should be created");
        PermissionService::update_permissions_for_role(
            &ctx,
            UpdateRolePermissionsInput {
                site_id: site.site_id,
                role_reference: Reference::Id(everyone.role_id),
                new_permissions: [
                    Action::View,
                    Action::Create,
                    Action::Edit,
                    Action::Delete,
                ]
                .into_iter()
                .map(|action| Permission {
                    resource_type: Resource::Page,
                    resource_category: None,
                    action,
                })
                .collect(),
                cascade_removals: false,
                updating_user_id: SYSTEM_USER_ID,
                ip_address: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 61)),
            },
        )
        .await
        .expect("editable page role permissions should be granted");
        site.site_id
    };
    transaction
        .commit()
        .await
        .expect("editable site fixture transaction should commit");
    site_id
}

async fn rpc_request(
    client: &reqwest::Client,
    address: SocketAddr,
    session_token: Option<&str>,
    site_id: Option<i64>,
    page: Option<&str>,
    method: &str,
    params: Value,
) -> Value {
    let response = rpc_page_response(
        client,
        address,
        session_token,
        site_id,
        page,
        method,
        params,
    )
    .await;
    assert!(
        response.get("error").is_none(),
        "public Deepwell method {method} failed: {response}",
    );
    response
        .get("result")
        .cloned()
        .expect("public Deepwell response should contain a result")
}

async fn rpc_page_response(
    client: &reqwest::Client,
    address: SocketAddr,
    session_token: Option<&str>,
    site_id: Option<i64>,
    page: Option<&str>,
    method: &str,
    params: Value,
) -> Value {
    let mut request = client
        .post(format!("http://{address}"))
        .bearer_auth(
            env::var("DEEPWELL_RPC_TOKEN").expect("test RPC token must be configured"),
        )
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1061,
            "method": method,
            "params": params,
        }));
    if let Some(session_token) = session_token {
        request = request.header("X-Deepwell-Session-Token", session_token);
    }
    if let Some(site_id) = site_id {
        request = request.header("X-Deepwell-Site-Id", site_id);
    }
    if let Some(page) = page {
        request = request.header("X-Deepwell-Page", page);
    }

    request
        .send()
        .await
        .expect("public Deepwell request should complete")
        .json()
        .await
        .expect("public Deepwell response should be JSON")
}

async fn add_duplicate_and_cycle_connections(
    state: &ServerState,
    component: &PageModel,
    dependent: &PageModel,
) {
    let transaction = state
        .database
        .begin()
        .await
        .expect("dependency fixture transaction should begin");
    let existing = PageConnection::find()
        .filter(page_connection::Column::FromPageId.eq(dependent.page_id))
        .filter(page_connection::Column::ToPageId.eq(component.page_id))
        .all(&transaction)
        .await
        .expect("recorded component dependency should be readable");
    assert_eq!(
        existing.len(),
        1,
        "the authored include should establish one real dependency edge",
    );
    let duplicate_type = [
        ConnectionType::IncludeMessy,
        ConnectionType::IncludeElements,
        ConnectionType::Component,
    ]
    .into_iter()
    .find(|candidate| {
        existing
            .iter()
            .all(|connection| connection.connection_type != *candidate)
    })
    .expect("a second supported include connection type should exist");
    let now = OffsetDateTime::now_utc();
    page_connection::ActiveModel {
        from_page_id: Set(dependent.page_id),
        to_page_id: Set(component.page_id),
        connection_type: Set(duplicate_type),
        created_at: Set(now),
        updated_at: Set(None),
        count: Set(1),
    }
    .insert(&transaction)
    .await
    .expect("duplicate logical component edge should be inserted");
    page_connection::ActiveModel {
        from_page_id: Set(component.page_id),
        to_page_id: Set(dependent.page_id),
        connection_type: Set(ConnectionType::IncludeMessy),
        created_at: Set(now),
        updated_at: Set(None),
        count: Set(1),
    }
    .insert(&transaction)
    .await
    .expect("cycle fence relation should be inserted");
    transaction
        .commit()
        .await
        .expect("dependency fixture transaction should commit");
}

async fn assert_rollback_and_cycle_fences(
    state: &ServerState,
    rsmq: &mut Rsmq,
    component: &PageModel,
) {
    let rollback = state
        .database
        .begin()
        .await
        .expect("rollback assertion transaction should begin");
    {
        let ctx = ServiceContext::new(state, &rollback);
        OutdateService::outdate_outgoing_includes(
            &ctx,
            component.page_id,
            RerenderDepth::default(),
        )
        .await
        .expect("dependent discovery should succeed before rollback");
        assert_eq!(
            queued_job_count(rsmq).await,
            0,
            "A1061_POST_COMMIT_ROLLBACK: uncommitted outdating must not reach Redis",
        );
    }
    rollback
        .rollback()
        .await
        .expect("outdating fixture should roll back");
    assert_eq!(
        queued_job_count(rsmq).await,
        0,
        "A1061_POST_COMMIT_ROLLBACK: rollback must leave no queued rerender",
    );

    let terminal_depth = state
        .config
        .rerender_skip
        .iter()
        .find_map(|(depth, offset)| offset.is_none().then_some(*depth))
        .expect("integration config should define a terminal cycle fence");
    let cycle = state
        .database
        .begin()
        .await
        .expect("cycle fence transaction should begin");
    {
        let ctx = ServiceContext::new(state, &cycle);
        PageRevisionService::rerender(
            &ctx,
            PageId::from_page_model(component),
            RerenderDepth(terminal_depth),
            RerenderType::Full,
        )
        .await
        .expect("terminal rerender depth should stop a cyclic graph safely");
        assert_eq!(
            queued_job_count(rsmq).await,
            0,
            "A1061_DEPENDENCY_CYCLE_FENCE: terminal depth must not enqueue another layer",
        );
    }
    cycle
        .rollback()
        .await
        .expect("cycle fence fixture should roll back");
}

const SITE_NAVIGATION_TEST_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(192, 0, 2, 71));

struct SiteNavigationRpcFixture {
    state: ServerState,
    redis_url: String,
    namespace: String,
    queue: Rsmq,
    session_token: String,
    site: SiteModel,
    site_user_id: i64,
    role_id: i64,
    top_inheriting_page_ids: HashSet<i64>,
    side_inheriting_page_ids: HashSet<i64>,
}

impl SiteNavigationRpcFixture {
    async fn setup(redis_url: String, namespace: String) -> Self {
        let state = build_server_state_without_workers_with_job_queue_namespace(
            Config::integration_testing(),
            Secrets::load(),
            &namespace,
        )
        .await
        .expect("isolated site navigation state should build");
        let mut queue = Rsmq::clone(&state.rsmq);
        clear_job_queue(&mut queue).await;

        let transaction = state
            .database
            .begin()
            .await
            .expect("site navigation fixture transaction should begin");
        let ctx = ServiceContext::new(&state, &transaction);
        let run_id = Uuid::new_v4().simple().to_string();
        let created_site = SiteService::create(
            &ctx,
            CreateSite {
                slug: format!("site-nav-rpc-{run_id}"),
                name: String::from("Site navigation RPC fixture"),
                tagline: String::new(),
                description: String::from("UUID-owned navigation test state"),
                default_page: None,
                layout: Some(ftml::layout::Layout::Wikidot),
                license: License::CcBySa40,
                locale: String::from("en"),
                ip_address: SITE_NAVIGATION_TEST_IP,
            },
            None,
        )
        .await
        .expect("owned site navigation site should be created");
        let site = SiteService::get(&ctx, Reference::Id(created_site.site_id))
            .await
            .expect("owned site navigation site should load");

        let top_inheriting_category =
            CategoryService::get_or_create(&ctx, site.site_id, "top-inheriting")
                .await
                .expect("top-inheriting category should be created");
        let side_inheriting_category =
            CategoryService::get_or_create(&ctx, site.site_id, "side-inheriting")
                .await
                .expect("side-inheriting category should be created");
        page_category::ActiveModel {
            category_id: Set(top_inheriting_category.category_id),
            top_bar_page: Set(None),
            side_bar_page: Set(Some(String::from("fixture:side-override"))),
            ..Default::default()
        }
        .update(&transaction)
        .await
        .expect("top-inheriting category should be configured");
        page_category::ActiveModel {
            category_id: Set(side_inheriting_category.category_id),
            top_bar_page: Set(Some(String::from("fixture:top-override"))),
            side_bar_page: Set(None),
            ..Default::default()
        }
        .update(&transaction)
        .await
        .expect("side-inheriting category should be configured");

        let everyone = RoleService::create(
            &ctx,
            InternalCreateRoleInput {
                site_id: site.site_id,
                name: SystemRole::Everyone.to_string(),
                description: Some(String::from("Owned site-update RPC permission")),
                is_virtual: true,
                parent_role_id: None,
                creating_user_id: SYSTEM_USER_ID,
                ip_address: SITE_NAVIGATION_TEST_IP,
            },
        )
        .await
        .expect("owned everyone role should be created");
        PermissionService::update_permissions_for_role(
            &ctx,
            UpdateRolePermissionsInput {
                site_id: site.site_id,
                role_reference: Reference::Id(everyone.role_id),
                new_permissions: vec![Permission {
                    resource_type: Resource::Site,
                    resource_category: None,
                    action: Action::Edit,
                }],
                cascade_removals: false,
                updating_user_id: SYSTEM_USER_ID,
                ip_address: SITE_NAVIGATION_TEST_IP,
            },
        )
        .await
        .expect("owned site-edit permission should be granted");

        let mut top_inheriting_page_ids = HashSet::new();
        let mut side_inheriting_page_ids = HashSet::new();
        for index in 0..2 {
            let top_page = page::ActiveModel {
                site_id: Set(site.site_id),
                page_category_id: Set(top_inheriting_category.category_id),
                slug: Set(format!("top-inheriting-{run_id}-{index}")),
                ..Default::default()
            }
            .insert(&transaction)
            .await
            .expect("top-inheriting page should be created");
            top_inheriting_page_ids.insert(top_page.page_id);

            let side_page = page::ActiveModel {
                site_id: Set(site.site_id),
                page_category_id: Set(side_inheriting_category.category_id),
                slug: Set(format!("side-inheriting-{run_id}-{index}")),
                ..Default::default()
            }
            .insert(&transaction)
            .await
            .expect("side-inheriting page should be created");
            side_inheriting_page_ids.insert(side_page.page_id);
        }

        let session_token = SessionService::create(
            &ctx,
            CreateSession {
                user_id: ADMIN_USER_ID,
                ip_address: SITE_NAVIGATION_TEST_IP,
                user_agent: String::from("site navigation post-commit RPC test"),
                restricted: false,
            },
        )
        .await
        .expect("site navigation admin session should be created");
        transaction
            .commit()
            .await
            .expect("site navigation fixture should commit");

        Self {
            state,
            redis_url,
            namespace,
            queue,
            session_token,
            site,
            site_user_id: created_site.site_user_id,
            role_id: everyone.role_id,
            top_inheriting_page_ids,
            side_inheriting_page_ids,
        }
    }

    async fn cleanup(self) {
        let database_result = AssertUnwindSafe(self.cleanup_database())
            .catch_unwind()
            .await;
        let redis_result =
            AssertUnwindSafe(cleanup_namespace(&self.redis_url, &self.namespace))
                .catch_unwind()
                .await;
        let owned_redis_result = AssertUnwindSafe(self.cleanup_owned_redis_keys())
            .catch_unwind()
            .await;

        if let Err(payload) = database_result {
            std::panic::resume_unwind(payload);
        }
        if let Err(payload) = redis_result {
            std::panic::resume_unwind(payload);
        }
        if let Err(payload) = owned_redis_result {
            std::panic::resume_unwind(payload);
        }
    }

    async fn cleanup_database(&self) {
        let transaction = self
            .state
            .database
            .begin()
            .await
            .expect("site navigation cleanup transaction should begin");

        SessionTable::delete_by_id(self.session_token.clone())
            .exec(&transaction)
            .await
            .expect("owned site navigation session should be removed");
        PageTable::delete_many()
            .filter(page::Column::SiteId.eq(self.site.site_id))
            .exec(&transaction)
            .await
            .expect("owned site navigation pages should be removed");
        PageCategory::delete_many()
            .filter(page_category::Column::SiteId.eq(self.site.site_id))
            .exec(&transaction)
            .await
            .expect("owned site navigation categories should be removed");
        RolePermissionTable::delete_many()
            .filter(role_permission::Column::SiteId.eq(self.site.site_id))
            .exec(&transaction)
            .await
            .expect("owned site permissions should be removed");
        RoleTable::delete_many()
            .filter(role::Column::SiteId.eq(self.site.site_id))
            .exec(&transaction)
            .await
            .expect("owned site roles should be removed");
        RelationTable::delete_many()
            .filter(
                Condition::any()
                    .add(
                        Condition::all()
                            .add(relation::Column::DestType.eq(RelationObjectType::Site))
                            .add(relation::Column::DestId.eq(self.site.site_id)),
                    )
                    .add(
                        Condition::all()
                            .add(relation::Column::FromType.eq(RelationObjectType::User))
                            .add(relation::Column::FromId.eq(self.site_user_id)),
                    ),
            )
            .exec(&transaction)
            .await
            .expect("owned site-user relation should be removed");
        AuditLog::delete_many()
            .filter(
                Condition::any()
                    .add(audit_log::Column::SiteId.eq(self.site.site_id))
                    .add(audit_log::Column::UserId.eq(self.site_user_id))
                    .add(
                        Condition::all()
                            .add(
                                audit_log::Column::EventType
                                    .eq("role.update_permissions"),
                            )
                            .add(audit_log::Column::ExtraId1.eq(self.role_id)),
                    ),
            )
            .exec(&transaction)
            .await
            .expect("owned site navigation audit rows should be removed");
        UserTable::delete_by_id(self.site_user_id)
            .exec(&transaction)
            .await
            .expect("owned site user should be removed");
        SiteTable::delete_by_id(self.site.site_id)
            .exec(&transaction)
            .await
            .expect("owned site should be removed");
        KnownUserTable::delete_by_id(self.site_user_id)
            .exec(&transaction)
            .await
            .expect("owned known-user identity should be removed");
        transaction
            .commit()
            .await
            .expect("site navigation cleanup should commit");
    }

    async fn cleanup_owned_redis_keys(&self) {
        let public_cache_key = PublicContentCache::site_version_key(self.site.site_id);
        let permission_pattern = format!("permission:site:{}:*", self.site.site_id);
        let mut redis = self.state.redis.clone();
        let mut keys: Vec<String> = redis
            .keys(permission_pattern)
            .await
            .expect("owned permission cache keys should be listed");
        keys.push(public_cache_key);
        let _: usize = redis
            .del(keys)
            .await
            .expect("owned site cache keys should be removed");
    }
}

async fn rpc_response(
    client: &reqwest::Client,
    address: SocketAddr,
    session_token: &str,
    site_id: i64,
    method: &str,
    params: Value,
) -> Value {
    client
        .post(format!("http://{address}"))
        .bearer_auth(
            env::var("DEEPWELL_RPC_TOKEN").expect("test RPC token must be configured"),
        )
        .header("X-Deepwell-Session-Token", session_token)
        .header("X-Deepwell-Site-Id", site_id)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1174,
            "method": method,
            "params": params,
        }))
        .send()
        .await
        .expect("site navigation RPC should complete")
        .json()
        .await
        .expect("site navigation RPC should return JSON")
}

async fn drain_jobs(rsmq: &mut Rsmq) -> Vec<Job> {
    let mut jobs = Vec::new();
    while let Some(message) = rsmq
        .receive_message::<Vec<u8>>(JOB_QUEUE_NAME, None)
        .await
        .expect("site navigation queue should be readable")
    {
        jobs.push(
            serde_json::from_slice(&message.message)
                .expect("site navigation job should decode"),
        );
        rsmq.delete_message(JOB_QUEUE_NAME, &message.id)
            .await
            .expect("site navigation job should be removed");
    }
    jobs
}

fn assert_navigation_jobs(jobs: &[Job], expected_page_ids: &HashSet<i64>) {
    let actual_page_ids = jobs
        .iter()
        .map(|job| match job {
            Job::RerenderPage {
                id,
                r#type: RerenderType::NavigationOnly,
                ..
            } => id.page_id,
            other => panic!("expected navigation-only rerender, got {other:?}"),
        })
        .collect::<HashSet<_>>();
    assert_eq!(actual_page_ids, *expected_page_ids);
    assert_eq!(jobs.len(), expected_page_ids.len());
}

async fn run_site_navigation_rpc_contract(
    fixture: &mut SiteNavigationRpcFixture,
    address: SocketAddr,
) {
    let client = reqwest::Client::new();
    let site_id = fixture.site.site_id;
    let mut revision = fixture.site.settings_revision;

    let stale = rpc_response(
        &client,
        address,
        &fixture.session_token,
        site_id,
        "site_update",
        json!({
            "site": site_id,
            "user_id": SYSTEM_USER_ID,
            "expected_settings_revision": revision - 1,
            "top_bar_page": "fixture:stale-top",
            "ip_address": SITE_NAVIGATION_TEST_IP,
        }),
    )
    .await;
    assert!(
        stale.get("error").is_some(),
        "stale update should fail: {stale}"
    );
    assert!(drain_jobs(&mut fixture.queue).await.is_empty());

    let unchanged = rpc_response(
        &client,
        address,
        &fixture.session_token,
        site_id,
        "site_update",
        json!({
            "site": site_id,
            "user_id": SYSTEM_USER_ID,
            "expected_settings_revision": revision,
            "top_bar_page": fixture.site.top_bar_page,
            "ip_address": SITE_NAVIGATION_TEST_IP,
        }),
    )
    .await;
    assert!(
        unchanged.get("error").is_none(),
        "unchanged update should succeed: {unchanged}",
    );
    revision += 1;
    assert!(drain_jobs(&mut fixture.queue).await.is_empty());

    let side_only = rpc_response(
        &client,
        address,
        &fixture.session_token,
        site_id,
        "site_update",
        json!({
            "site": site_id,
            "user_id": SYSTEM_USER_ID,
            "expected_settings_revision": revision,
            "side_bar_page": "fixture:new-side",
            "ip_address": SITE_NAVIGATION_TEST_IP,
        }),
    )
    .await;
    assert!(
        side_only.get("error").is_none(),
        "side-only update should succeed: {side_only}",
    );
    revision += 1;
    assert_eq!(side_only["result"]["side_bar_page"], "fixture:new-side");
    assert_eq!(
        side_only["result"]["top_bar_page"],
        fixture.site.top_bar_page,
    );
    assert_navigation_jobs(
        &drain_jobs(&mut fixture.queue).await,
        &fixture.side_inheriting_page_ids,
    );

    let both = rpc_response(
        &client,
        address,
        &fixture.session_token,
        site_id,
        "site_update",
        json!({
            "site": site_id,
            "user_id": SYSTEM_USER_ID,
            "expected_settings_revision": revision,
            "top_bar_page": "fixture:new-top",
            "side_bar_page": "fixture:new-side-again",
            "ip_address": SITE_NAVIGATION_TEST_IP,
        }),
    )
    .await;
    assert!(
        both.get("error").is_none(),
        "combined navigation update should succeed: {both}",
    );
    assert_eq!(both["result"]["top_bar_page"], "fixture:new-top");
    assert_eq!(both["result"]["side_bar_page"], "fixture:new-side-again",);
    let expected_both = fixture
        .top_inheriting_page_ids
        .union(&fixture.side_inheriting_page_ids)
        .copied()
        .collect();
    assert_navigation_jobs(&drain_jobs(&mut fixture.queue).await, &expected_both);

    let fetched = rpc_response(
        &client,
        address,
        &fixture.session_token,
        site_id,
        "site_get",
        json!({ "site": site_id }),
    )
    .await;
    assert_eq!(fetched["result"]["top_bar_page"], "fixture:new-top");
    assert_eq!(fetched["result"]["side_bar_page"], "fixture:new-side-again",);
}

#[tokio::test]
async fn site_navigation_updates_persist_and_dispatch_after_registered_rpc_commit() {
    let redis_url =
        env::var("REDIS_URL").expect("REDIS_URL must be set for integration tests");
    let namespace = format!("site-navigation-rpc-{}", Uuid::new_v4().simple(),);
    let setup = AssertUnwindSafe(SiteNavigationRpcFixture::setup(
        redis_url.clone(),
        namespace.clone(),
    ))
    .catch_unwind()
    .await;
    let mut fixture = match setup {
        Ok(fixture) => fixture,
        Err(payload) => {
            cleanup_namespace(&redis_url, &namespace).await;
            std::panic::resume_unwind(payload);
        }
    };
    let server = build_server_at(
        fixture.state.clone(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
    )
    .await;
    let (address, handle) = match server {
        Ok(server) => server,
        Err(error) => {
            fixture.cleanup().await;
            panic!("site navigation RPC server should start: {error:?}");
        }
    };

    let result =
        AssertUnwindSafe(run_site_navigation_rpc_contract(&mut fixture, address))
            .catch_unwind()
            .await;
    let stop_result = AssertUnwindSafe(async {
        handle
            .stop()
            .expect("site navigation RPC server should stop");
        handle.stopped().await;
    })
    .catch_unwind()
    .await;
    fixture.cleanup().await;

    if let Err(payload) = result {
        std::panic::resume_unwind(payload);
    }
    if let Err(payload) = stop_result {
        std::panic::resume_unwind(payload);
    }
}

#[tokio::test]
async fn component_css_save_refreshes_direct_dependent_before_rpc_returns() {
    let redis_url =
        env::var("REDIS_URL").expect("REDIS_URL must be set for integration tests");
    let run_id = Uuid::new_v4().simple().to_string();
    let namespace = format!("rsmq-authoring-1061-{run_id}");
    let mut config = Config::integration_testing();
    config.job_min_poll_delay = Duration::from_millis(10);
    config.job_max_poll_delay = Duration::from_millis(50);
    config.job_work_delay = Duration::from_millis(10);
    config.rerender_skip = vec![(10, None)];
    let state = build_server_state_without_workers_with_job_queue_namespace(
        config,
        Secrets::load(),
        &namespace,
    )
    .await
    .expect("isolated post-commit worker state should build");
    let mut rsmq = Rsmq::clone(&state.rsmq);
    clear_job_queue(&mut rsmq).await;

    let site_slug = format!("job-queue-1061-{run_id}");
    let fixture_site_id = create_editable_site_fixture(&state, &site_slug).await;
    let session_token = create_admin_session(&state).await;
    let (address, handle) = build_server_at(
        state.clone(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
    )
    .await
    .expect("public Deepwell server should start");
    let client = reqwest::Client::new();
    let site = rpc_request(
        &client,
        address,
        None,
        None,
        None,
        "site_get",
        json!({"site": site_slug}),
    )
    .await;
    let site_id = site["site_id"]
        .as_i64()
        .expect("editable site ID should be present");
    assert_eq!(site_id, fixture_site_id);
    let component_slug = format!("component:authoring-post-commit-{run_id}");
    let dependent_slug = format!("authoring-post-commit-dependent-{run_id}");
    let red_css = "[[module CSS]]\n.authoring-color { color: red; }\n[[/module]]";
    let blue_css = "[[module CSS]]\n.authoring-color { color: blue; }\n[[/module]]";
    let green_css = "[[module CSS]]\n.authoring-color { color: green; }\n[[/module]]";

    let component = rpc_request(
        &client,
        address,
        Some(&session_token),
        Some(site_id),
        Some(&component_slug),
        "page_create",
        json!({
            "site_id": site_id,
            "wikitext": red_css,
            "title": "Authoring post-commit component",
            "alt_title": null,
            "slug": component_slug,
            "layout": "wikidot",
            "revision_comments": "create issue 1061 component",
            "user_id": ADMIN_USER_ID,
            "ip_address": "192.0.2.61",
        }),
    )
    .await;
    let component_id = component["page_id"]
        .as_i64()
        .expect("component page ID should be present");
    let component_revision_id = component["revision_id"]
        .as_i64()
        .expect("component revision ID should be present");
    let dependent = rpc_request(
        &client,
        address,
        Some(&session_token),
        Some(site_id),
        Some(&dependent_slug),
        "page_create",
        json!({
            "site_id": site_id,
            "wikitext": format!("[[include {component_slug}]]\nDependent body"),
            "title": "Authoring post-commit dependent",
            "alt_title": null,
            "slug": dependent_slug,
            "layout": "wikidot",
            "revision_comments": "create issue 1061 dependent",
            "user_id": ADMIN_USER_ID,
            "ip_address": "192.0.2.61",
        }),
    )
    .await;
    let dependent_id = dependent["page_id"]
        .as_i64()
        .expect("dependent page ID should be present");
    clear_job_queue(&mut rsmq).await;

    let fixture = state
        .database
        .begin()
        .await
        .expect("page identity fixture transaction should begin");
    let component_page = PageTable::find_by_id(component_id)
        .one(&fixture)
        .await
        .expect("component page lookup should succeed")
        .expect("component page should exist");
    let dependent_page = PageTable::find_by_id(dependent_id)
        .one(&fixture)
        .await
        .expect("dependent page lookup should succeed")
        .expect("dependent page should exist");
    fixture
        .rollback()
        .await
        .expect("page identity fixture transaction should roll back");

    add_duplicate_and_cycle_connections(&state, &component_page, &dependent_page).await;
    assert_rollback_and_cycle_fences(&state, &mut rsmq, &component_page).await;

    let before = rpc_request(
        &client,
        address,
        None,
        Some(site_id),
        Some(&dependent_slug),
        "article_view",
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": dependent_slug, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    )
    .await;
    assert!(
        before["page"]["data"]["compiled_body_styles"]
            .as_array()
            .is_some_and(|styles| styles.iter().any(|style| {
                style
                    .as_str()
                    .is_some_and(|style| style.contains("color: red"))
            })),
        "dependent should serve the original red component CSS before save",
    );

    let component_page_header = component_id.to_string();
    let blue_edit = rpc_page_response(
        &client,
        address,
        Some(&session_token),
        Some(site_id),
        Some(&component_page_header),
        "page_edit",
        json!({
            "site_id": site_id,
            "page": component_id,
            "last_revision_id": component_revision_id,
            "revision_comments": "change issue 1061 component to blue",
            "user_id": ADMIN_USER_ID,
            "wikitext": blue_css,
            "ip_address": "192.0.2.61",
        }),
    );
    let green_edit = rpc_page_response(
        &client,
        address,
        Some(&session_token),
        Some(site_id),
        Some(&component_page_header),
        "page_edit",
        json!({
            "site_id": site_id,
            "page": component_id,
            "last_revision_id": component_revision_id,
            "revision_comments": "change issue 1061 component to green",
            "user_id": ADMIN_USER_ID,
            "wikitext": green_css,
            "ip_address": "192.0.2.61",
        }),
    );
    let (blue_response, green_response) = tokio::join!(blue_edit, green_edit);
    let blue_succeeded =
        blue_response.get("result").is_some() && blue_response.get("error").is_none();
    let green_succeeded =
        green_response.get("result").is_some() && green_response.get("error").is_none();
    assert_ne!(
        blue_succeeded, green_succeeded,
        "A1061_CONCURRENT_COMPONENT_SAVE: exactly one edit from the same source revision must commit; blue={blue_response} green={green_response}",
    );
    let winning_color = if blue_succeeded { "blue" } else { "green" };
    let losing_color = if blue_succeeded { "green" } else { "blue" };
    assert_eq!(
        queued_job_count(&mut rsmq).await,
        0,
        "A1061_POST_COMMIT_DISPATCH: the direct dependent should finish before page_edit returns",
    );

    let first_after_save = rpc_request(
        &client,
        address,
        None,
        Some(site_id),
        Some(&dependent_slug),
        "article_view",
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": dependent_slug, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    )
    .await;
    let styles = first_after_save["page"]["data"]["compiled_body_styles"]
        .as_array()
        .expect("served dependent styles should be an array");
    assert!(
        styles.iter().any(|style| {
            style
                .as_str()
                .is_some_and(|style| style.contains(&format!("color: {winning_color}")))
        }),
        "the first article_view after concurrent component save should contain only the winning CSS",
    );
    assert!(
        styles.iter().all(|style| {
            style.as_str().is_none_or(|style| {
                !style.contains("color: red")
                    && !style.contains(&format!("color: {losing_color}"))
            })
        }),
        "the first article_view after concurrent component save must contain neither stale nor losing CSS",
    );

    handle.stop().expect("public Deepwell server should stop");
    handle.stopped().await;
    cleanup_namespace(&redis_url, &namespace).await;
}

async fn served_body(
    client: &reqwest::Client,
    address: SocketAddr,
    site_id: i64,
    slug: &str,
) -> String {
    let view = rpc_request(
        client,
        address,
        None,
        Some(site_id),
        Some(slug),
        "article_view",
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": slug, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    )
    .await;
    view["page"]["data"]["compiled_body_html"]
        .as_str()
        .expect("page body should be served")
        .to_owned()
}

/// Wikidot re-renders pages that referenced a slug while no page existed
/// there as soon as one is created, and leaves the pages that include or link
/// a page untouched when it is deleted (live sandbox-for-codex observations,
/// 2026-10-15, run-owned `wjctl1015-*` / `wjctk1015-*` pages at 2-120 s).
#[tokio::test]
async fn created_target_refreshes_referrers_and_deleted_target_leaves_them() {
    let redis_url =
        env::var("REDIS_URL").expect("REDIS_URL must be set for integration tests");
    let run_id = Uuid::new_v4().simple().to_string();
    let namespace = format!("rsmq-absent-target-{run_id}");
    let mut config = Config::integration_testing();
    config.job_min_poll_delay = Duration::from_millis(10);
    config.job_max_poll_delay = Duration::from_millis(50);
    config.job_work_delay = Duration::from_millis(10);
    config.rerender_skip = vec![(10, None)];
    let state = build_server_state_without_workers_with_job_queue_namespace(
        config,
        Secrets::load(),
        &namespace,
    )
    .await
    .expect("isolated post-commit worker state should build");
    let mut rsmq = Rsmq::clone(&state.rsmq);
    clear_job_queue(&mut rsmq).await;

    let site_slug = format!("absent-target-{run_id}");
    let site_id = create_editable_site_fixture(&state, &site_slug).await;
    let session_token = create_admin_session(&state).await;
    let (address, handle) = build_server_at(
        state.clone(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
    )
    .await
    .expect("public Deepwell server should start");
    let client = reqwest::Client::new();
    let target_slug = format!("absent-target-page-{run_id}");
    let holder_slug = format!("absent-target-holder-{run_id}");

    rpc_request(
        &client,
        address,
        Some(&session_token),
        Some(site_id),
        Some(&holder_slug),
        "page_create",
        json!({
            "site_id": site_id,
            "wikitext": format!(
                "BEFORE\n[[include {target_slug} a=present]]\n[[[{target_slug}|to target]]]\nAFTER",
            ),
            "title": "Absent target holder",
            "alt_title": null,
            "slug": holder_slug,
            "layout": "wikidot",
            "revision_comments": "create holder of an absent target",
            "user_id": ADMIN_USER_ID,
            "ip_address": "192.0.2.62",
        }),
    )
    .await;
    let newpage_link = format!("class=\"newpage\" href=\"/{target_slug}\"");

    let before = served_body(&client, address, site_id, &holder_slug).await;
    assert!(before.contains("does not exist"), "{before}");
    assert!(before.contains(&newpage_link), "{before}");

    let target = rpc_request(
        &client,
        address,
        Some(&session_token),
        Some(site_id),
        Some(&target_slug),
        "page_create",
        json!({
            "site_id": site_id,
            "wikitext": "TARGET-EXISTS a=[{$a}]",
            "title": "Absent target",
            "alt_title": null,
            "slug": target_slug,
            "layout": "wikidot",
            "revision_comments": "create the absent target",
            "user_id": ADMIN_USER_ID,
            "ip_address": "192.0.2.62",
        }),
    )
    .await;
    assert_eq!(
        queued_job_count(&mut rsmq).await,
        0,
        "the referrer should refresh before page_create returns",
    );
    let created = served_body(&client, address, site_id, &holder_slug).await;
    assert!(created.contains("TARGET-EXISTS a=[present]"), "{created}");
    assert!(!created.contains("does not exist"), "{created}");
    assert!(!created.contains(&newpage_link), "{created}");

    rpc_request(
        &client,
        address,
        Some(&session_token),
        Some(site_id),
        Some(&target_slug),
        "page_delete",
        json!({
            "site_id": site_id,
            "page": target["page_id"],
            "last_revision_id": target["revision_id"],
            "revision_comments": "delete the target",
            "user_id": ADMIN_USER_ID,
            "ip_address": "192.0.2.62",
        }),
    )
    .await;
    let deleted = served_body(&client, address, site_id, &holder_slug).await;
    assert_eq!(
        deleted, created,
        "Wikidot keeps serving the deleted target's content in pages that include it",
    );

    handle.stop().expect("public Deepwell server should stop");
    handle.stopped().await;
    cleanup_namespace(&redis_url, &namespace).await;
}
