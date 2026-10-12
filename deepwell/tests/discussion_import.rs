#[macro_use]
mod common;

use common::TestRunner;
use deepwell::constants::ADMIN_USER_ID;
use deepwell::models::{forum_post, forum_post_revision, page, user, wikidot_user};
use deepwell::services::{RequestContext, TextService};
use sea_orm::{
    ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, Statement,
    Value,
};
use serde_json::{Value as JsonValue, json};
use uuid::Uuid;

async fn fixture(runner: &mut TestRunner) -> JsonValue {
    let site_id = run_endpoint!(runner, site_get, json!({"site":"test"}))
        .unwrap()
        .site
        .site_id;
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        ..Default::default()
    });
    let tag = Uuid::new_v4();
    let page = run_endpoint!(
        runner,
        page_create,
        json!({"site_id":site_id,"wikitext":"Imported discussion fixture","title":"Imported discussion fixture","alt_title":null,"slug":format!("discussion-import-{tag}"),"layout":"wikidot","revision_comments":"fixture","user_id":ADMIN_USER_ID,"bypass_filter":true,"ip_address":common::IP_ADDRESS})
    );
    let db = runner.context().transaction();
    db.execute_raw(Statement::from_sql_and_values(
        db.get_database_backend(),
        "UPDATE page SET from_wikidot=true WHERE page_id=$1",
        [Value::from(page.page_id)],
    ))
    .await
    .unwrap();
    let import=db.query_one_raw(Statement::from_sql_and_values(db.get_database_backend(),"INSERT INTO wikidot_corpus_import_run(site_id,source_branch,source_site,manifest_sha256,manifest_row_count,complete_inventory,state) VALUES($1,'en','source-site',decode(repeat('ab',32),'hex'),1,false,'done') RETURNING import_run_id",[Value::from(site_id)])).await.unwrap().unwrap().try_get::<i64>("","import_run_id").unwrap();
    db.execute_raw(Statement::from_sql_and_values(db.get_database_backend(),"INSERT INTO wikidot_page_snapshot(page_id,source_branch,source_site,source_entity_id,source_fullname,source_created_at,source_updated_at,source_revision_count,imported_rating,comments,source_sha256,meta_sha256,meta_json,last_import_run_id) VALUES($1,'en','source-site',$2::uuid,$4,NOW(),NOW(),1,0,2,decode(repeat('ab',32),'hex'),decode(repeat('cd',32),'hex'),'{}',$3)",[Value::from(page.page_id),Value::from(tag.to_string()),Value::from(import),Value::from(format!("ordinary-page-{tag}"))])).await.unwrap();
    let first = 40_000_000
        + i64::from(
            u32::from_be_bytes(tag.as_bytes()[..4].try_into().unwrap()) % 1_000_000,
        ) * 10;
    json!({"site_id":site_id,"page_id":page.page_id,"source":{
        "site":"source-site","fullname":format!("ordinary-page-{tag}"),"reference":format!("https://source-site.wikidot.com/forum/t-{}/example",first+2),"evidence_sha256":"a".repeat(64),"observed_at":"2026-10-12T00:00:00Z",
        "category":{"id":first+1,"name":"Pages","description":"Page discussions","group_name":format!("Imported {tag}"),"group_description":"","group_visible":true,"group_sort_index":0,"sort_index":0},
        "thread":{"id":first+2,"title":"Example","description":"Page discussion","created_at":"2026-01-01T00:00:00Z","creator":{"type":"system"},"post_count":2},
        "actors":[{"id":first,"name":"Observed User","slug":format!("observed-{tag}")}],
        "posts":[{"id":first+3,"parent_id":null,"user_id":first,"created_at":"2026-01-01T00:00:00Z","edit":null,"title":"Root","wikitext":format!("Thanks [[user old-{tag}]]"),"referenced_user_ids":[first],"observed_users":[{"id":first,"name":"Old User","slug":format!("old-{tag}"),"avatar_url":null}]},
            {"id":first+4,"parent_id":first+3,"user_id":first,"created_at":"2026-01-02T00:00:00Z","edit":{"user_id":first,"created_at":"2026-01-03T00:00:00Z"},"title":"Reply","wikitext":"Original reply","referenced_user_ids":[]}]
    }})
}

#[tokio::test]
async fn imported_discussion_preserves_ids_appends_revisions_and_creates_no_account() {
    let mut runner = TestRunner::setup().await;
    let mut input = fixture(&mut runner).await;
    let source = &input["source"];
    let actor = source["actors"][0]["id"].as_i64().unwrap();
    let root = source["posts"][0]["id"].as_i64().unwrap();
    let reply = source["posts"][1]["id"].as_i64().unwrap();
    let first = run_endpoint!(runner, import_wikidot_discussion, input.clone());
    assert!(first.created_thread);
    assert_eq!(first.created_posts, 2);
    assert_eq!(first.created_revisions, 2);
    assert!(!first.unchanged);
    assert!(
        user::Entity::find_by_id(actor)
            .one(runner.context().transaction())
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        wikidot_user::Entity::find_by_id(actor as i32)
            .one(runner.context().transaction())
            .await
            .unwrap()
            .is_none()
    );
    let imported = page::Entity::find_by_id(input["page_id"].as_i64().unwrap())
        .one(runner.context().transaction())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(imported.discussion_thread_id, Some(first.forum_thread_id));
    let root_post = forum_post::Entity::find_by_id(root)
        .one(runner.context().transaction())
        .await
        .unwrap()
        .unwrap();
    let root_revision =
        forum_post_revision::Entity::find_by_id(root_post.latest_revision_id.unwrap())
            .one(runner.context().transaction())
            .await
            .unwrap()
            .unwrap();
    let html = TextService::get(runner.context(), &root_revision.compiled_html_hash)
        .await
        .unwrap();
    assert!(html.contains("Old User"));
    assert!(!html.contains("Observed User"));
    assert!(html.contains("printuser"));
    assert!(!html.contains("user does not exist"));
    let post = forum_post::Entity::find_by_id(reply)
        .one(runner.context().transaction())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(post.parent_post_id, Some(root));
    let db = runner.context().transaction();
    let source_edit = db.query_one_raw(Statement::from_sql_and_values(db.get_database_backend(),"SELECT source_has_edit FROM wikidot_discussion_post_snapshot WHERE forum_post_id=$1",[Value::from(reply)])).await.unwrap().unwrap().try_get::<bool>("","source_has_edit").unwrap();
    assert!(source_edit);
    let source_display = db
        .query_one_raw(Statement::from_sql_and_values(
            db.get_database_backend(),
            "SELECT name FROM wikidot_display_user WHERE user_id=$1",
            [Value::from(actor)],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get::<String>("", "name")
        .unwrap();
    assert_eq!(source_display, "Observed User");
    let original_revision = post.latest_revision_id.unwrap();
    let second = run_endpoint!(runner, import_wikidot_discussion, input.clone());
    assert_eq!(second.import_run_id, first.import_run_id);
    assert!(second.unchanged);
    assert_eq!(second.created_revisions, 0);
    input["source"]["observed_at"] = json!("2026-10-12T01:00:00Z");
    input["source"]["posts"][1]["wikitext"] = json!("Updated reply");
    input["source"]["posts"][1]["edit"]["created_at"] = json!("2026-10-12T00:30:00Z");
    input["source"]["category"]["description"] = json!("Updated source description");
    let changed = run_endpoint!(runner, import_wikidot_discussion, input.clone());
    assert_eq!(changed.created_posts, 0);
    assert_eq!(changed.created_revisions, 1);
    assert!(!changed.unchanged);
    assert!(
        forum_post_revision::Entity::find_by_id(original_revision)
            .one(runner.context().transaction())
            .await
            .unwrap()
            .is_some()
    );
    assert_eq!(
        forum_post_revision::Entity::find()
            .filter(forum_post_revision::Column::ForumPostId.eq(reply))
            .count(runner.context().transaction())
            .await
            .unwrap(),
        2
    );
    input["source"]["observed_at"] = json!("2026-10-11T00:00:00Z");
    let _ = run_endpoint_err!(runner, import_wikidot_discussion, input.clone());
    runner.set_request_context(RequestContext::default());
    let _ = run_endpoint_err!(runner, import_wikidot_discussion, input);
    runner.teardown().await;
}

#[tokio::test]
async fn discussion_import_rejects_foreign_threads_and_incomplete_graph_before_identity_writes()
 {
    let mut runner = TestRunner::setup().await;
    let first = fixture(&mut runner).await;
    let imported = run_endpoint!(runner, import_wikidot_discussion, first.clone());
    let mut second = fixture(&mut runner).await;
    let actor = second["source"]["actors"][0]["id"].as_i64().unwrap();
    second["source"]["thread"]["id"] = json!(imported.forum_thread_id);
    second["source"]["category"]["id"] = first["source"]["category"]["id"].clone();
    second["source"]["reference"] = json!(format!(
        "https://source-site.wikidot.com/forum/t-{}/example",
        imported.forum_thread_id
    ));
    let _ = run_endpoint_err!(runner, import_wikidot_discussion, second.clone());
    assert!(
        deepwell::models::known_user::Entity::find_by_id(actor)
            .one(runner.context().transaction())
            .await
            .unwrap()
            .is_none()
    );
    let page = page::Entity::find_by_id(second["page_id"].as_i64().unwrap())
        .one(runner.context().transaction())
        .await
        .unwrap()
        .unwrap();
    assert!(page.discussion_thread_id.is_none());
    second["source"]["posts"][1]["parent_id"] = json!(999999999);
    let _ = run_endpoint_err!(runner, import_wikidot_discussion, second.clone());
    assert!(
        deepwell::models::known_user::Entity::find_by_id(actor)
            .one(runner.context().transaction())
            .await
            .unwrap()
            .is_none()
    );
    runner.teardown().await;
}
