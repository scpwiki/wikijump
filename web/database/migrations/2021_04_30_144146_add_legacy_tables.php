<?php

use Database\Seeders\LegacySeeder;
use Database\Seeders\PHPUnitLegacySeeder;
use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Artisan;
use Illuminate\Support\Facades\Schema;

class AddLegacyTables extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        Schema::create('admin', function(Blueprint $table) {
            $table->id('admin_id')->startingValue(3);
            $table->unsignedInteger('site_id')->nullable();
            $table->unsignedInteger('user_id')->nullable();
            $table->boolean('founder')->nullable()->default(false);

            $table->unique(['site_id', 'user_id']);
        });

        Schema::create('admin_notification', function(Blueprint $table) {
           $table->id('notification_id');
           $table->unsignedInteger('site_id')->nullable()->index();
           $table->string('body')->nullable();
           $table->string('type', 50)->nullable();
           $table->boolean('viewed')->default(false)->nullable();
           $table->timestamp('date')->nullable();
           $table->binary('extra')->nullable();
           $table->boolean('notify_online')->default(false)->nullable();
           $table->boolean('notify_feed')->default(false)->nullable();
           $table->boolean('notify_email')->default(false)->nullable();
        });

        Schema::create('anonymous_abuse_flag', function(Blueprint $table) {
            $table->id('flag_id');
            $table->unsignedInteger('user_id')->nullable();
            $table->ipAddress('address')->nullable()->index();
            $table->boolean('proxy')->default(false)->nullable();
            $table->unsignedInteger('site_id')->nullable()->index();
            $table->boolean('site_valid')->default(true)->nullable();
            $table->boolean('global_valid')->default(true)->nullable();
        });

        Schema::create('category', function (Blueprint $table) {
           $table->id('category_id')->startingValue(22);
           $table->unsignedInteger('site_id')->nullable()->index();
           $table->string('name', 80)->nullable()->index();
           $table->boolean('theme_default')->default(true);
           $table->unsignedInteger('theme_id')->nullable();
           $table->boolean('permissions_default')->default(true);
           $table->string('permissions', 200)->nullable();
           $table->boolean('license_default')->default(true);
           $table->unsignedInteger('license_id')->nullable();
           $table->string('license_other', 350)->nullable();
           $table->boolean('nav_default')->default(true);
           $table->string('top_bar_page_name', 120)->nullable();
           $table->string('side_bar_page_name', 120)->nullable();
           $table->unsignedInteger('template_id')->nullable();
           $table->boolean('per_page_discussion')->nullable();
           $table->boolean('per_page_discussion_default')->default(true);
           $table->string('rating', 10)->nullable();
           $table->unsignedInteger('category_template_id')->nullable();
           $table->string('theme_external_url', 512)->nullable();
           $table->boolean('autonumerate')->default(false);
           $table->string('page_title_template', 256)->nullable();
        });

        Schema::create('category_template', function (Blueprint $table) {
           $table->id('category_template_id');
           $table->string('source', 200000);
        });

        Schema::create('comment', function (Blueprint $table) {
           $table->id('comment_id');
           $table->unsignedInteger('page_id')->nullable();
           $table->unsignedInteger('parent_id')->nullable();
           $table->unsignedInteger('user_id')->nullable();
           $table->string('user_string', 80)->nullable();
           $table->string('title', 256)->nullable();
           $table->string('text', 200000)->nullable();
           $table->timestamp('date_posted')->nullable();
           $table->unsignedInteger('site_id')->nullable();
           $table->unsignedInteger('revision_number')->default(0);
           $table->unsignedInteger('revision_id')->nullable();
           $table->timestamp('date_last_edited')->nullable();
           $table->unsignedInteger('edited_user_id')->nullable();
           $table->string('edited_user_string', 80)->nullable();
        });

        Schema::create('comment_revision', function (Blueprint $table) {
           $table->id('revision_id');
           $table->unsignedInteger('comment_id')->nullable();
           $table->unsignedInteger('user_id')->nullable();
           $table->string('user_string', 80)->nullable();
           $table->string('text', 200000)->nullable();
           $table->string('title', 256)->nullable();
           $table->timestamp('date')->nullable();
        });

        Schema::create('contact', function (Blueprint $table) {
           $table->id('contact_id');
           $table->unsignedInteger('user_id')->nullable();
           $table->unsignedInteger('target_user_id')->nullable();

           $table->unique(['user_id', 'target_user_id']);
        });

        Schema::create('domain_redirect', function (Blueprint $table) {
           $table->id('redirect_id');
           $table->unsignedInteger('site_id')->nullable();
           $table->string('url', 80)->nullable();

           $table->unique(['site_id', 'url']);
        });

        Schema::create('email_invitation', function(Blueprint $table) {
            $table->id('invitation_id');
            $table->string('hash', 200)->nullable();
            $table->string('email', 128)->nullable();
            $table->string('name', 100)->nullable();
            $table->unsignedInteger('user_id')->nullable()->index();
            $table->unsignedInteger('site_id')->nullable()->index();
            $table->boolean('become_member')->default(true);
            $table->boolean('to_contacts')->nullable();
            $table->string('message', 200000)->nullable();
            $table->unsignedInteger('attempts')->default(1);
            $table->boolean('accepted')->default(false);
            $table->boolean('delivered')->default(true);
            $table->timestamp('date')->nullable();
        });

        Schema::create('file', function (Blueprint $table) {
           $table->id('file_id');
           $table->unsignedInteger('page_id')->nullable()->index();
           $table->unsignedInteger('site_id')->nullable()->index();
           $table->string('filename', 100)->nullable();
           $table->string('mimetype', 100)->nullable();
           $table->string('description' ,200)->nullable();
           $table->string('description_short', 200)->nullable();
           $table->string('comment', 400)->nullable();
           $table->unsignedInteger('size')->nullable();
           $table->timestamp('date_added')->nullable();
           $table->unsignedInteger('user_id')->nullable();
           $table->string('user_string', 80)->nullable();
           $table->boolean('has_resized')->default(false);
        });

        Schema::create('files_event', function (Blueprint $table) {
           $table->id('file_event_id');
           $table->string('filename', 100)->nullable();
           $table->timestamp('date')->nullable();
           $table->unsignedInteger('user_id')->nullable();
           $table->string('user_string', 80)->nullable();
           $table->string('action', 80)->nullable();
           $table->string('action_extra', 80)->nullable();
        });

        Schema::create('forum_category', function (Blueprint $table) {
           $table->id('category_id');
           $table->unsignedInteger('group_id')->nullable()->index();
           $table->string('name', 80)->nullable();
           $table->string('description', 200000)->nullable();
           $table->unsignedInteger('number_posts')->default(0);
           $table->unsignedInteger('number_threads')->default(0);
           $table->unsignedInteger('last_post_id')->nullable()->index();
           $table->boolean('permissions_default')->default(true);
           $table->string('permissions', 200)->nullable();
           $table->unsignedInteger('max_nest_level')->nullable();
           $table->unsignedInteger('sort_index')->default(0);
           $table->unsignedInteger('site_id')->nullable()->index();
           $table->boolean('per_page_discussion')->default(false);
        });

        Schema::create('forum_group', function (Blueprint $table) {
           $table->id('group_id');
           $table->string('name', 80)->nullable();
           $table->string('description', 200000)->nullable();
           $table->unsignedInteger('sort_index')->default(0);
           $table->unsignedInteger('site_id')->nullable()->index();
           $table->boolean('visible')->default(true);
        });

        Schema::create('forum_post', function (Blueprint $table) {
           $table->id('post_id');
           $table->unsignedInteger('thread_id')->nullable()->index();
           $table->unsignedInteger('parent_id')->nullable();
           $table->unsignedInteger('user_id')->nullable()->index();
           $table->string('user_string', 80)->nullable();
           $table->string('title', 256)->nullable();
           $table->string('text', 200000)->nullable();
           $table->timestamp('date_posted')->nullable();
           $table->unsignedInteger('site_id')->nullable()->index();
           $table->unsignedInteger('revision_number')->default(0);
           $table->unsignedInteger('revision_id')->nullable();
           $table->timestamp('date_last_edited')->nullable();
           $table->unsignedInteger('edited_user_id')->nullable();
           $table->string('edited_user_string', 80)->nullable();
        });

        Schema::create('forum_post_revision', function (Blueprint $table) {
           $table->id('revision_id');
           $table->unsignedInteger('post_id')->nullable()->index();
           $table->unsignedInteger('user_id')->nullable();
           $table->string('user_string', 80)->nullable();
           $table->string('text', 200000)->nullable();
           $table->string('title', 256)->nullable();
           $table->timestamp('date')->nullable();
        });

        Schema::create('forum_settings', function (Blueprint $table) {
           $table->unsignedInteger('site_id')->primary();
           $table->string('permissions', 200)->nullable();
           $table->boolean('per_page_discussion')->default(false);
           $table->unsignedInteger('max_nest_level')->default(0);
        });

        Schema::create('forum_thread', function (Blueprint $table) {
           $table->id('thread_id');
           $table->unsignedInteger('user_id')->nullable()->index();
           $table->string('user_string', 80)->nullable();
           $table->unsignedInteger('category_id')->nullable()->index();
           $table->string('title', 256)->nullable();
           $table->string('description', 1000)->nullable();
           $table->unsignedInteger('number_posts')->default(1);
           $table->timestamp('date_started')->nullable();
           $table->unsignedInteger('site_id')->nullable()->index();
           $table->unsignedInteger('last_post_id')->nullable()->index();
           $table->unsignedInteger('page_id')->nullable()->index();
           $table->boolean('sticky')->default(false);
           $table->boolean('blocked')->default(false);
        });

        Schema::create('front_forum_feed', function (Blueprint $table) {
           $table->id('feed_id');
           $table->unsignedInteger('page_id')->nullable()->index();
           $table->string('title', 90)->nullable();
           $table->string('label', 90)->nullable();
           $table->string('description', 256)->nullable();
           $table->string('categories', 100)->nullable();
           $table->string('parmhash', 100)->nullable();
           $table->unsignedInteger('site_id')->nullable();
        });

        Schema::create('global_ip_block', function (Blueprint $table) {
           $table->id('block_id');
           $table->ipAddress('address')->nullable();
           $table->boolean('flag_proxy')->default(false);
           $table->string('reason', 200000)->nullable();
           $table->boolean('flag_total')->default(false);
           $table->timestamp('date_blocked')->nullable();
        });

        Schema::create('global_user_block', function (Blueprint $table) {
           $table->id('block_id');
           $table->unsignedInteger('site_id')->nullable();
           $table->unsignedInteger('user_id')->nullable();
           $table->string('reason', 200000)->nullable();
           $table->timestamp('date_blocked')->nullable();
        });

        Schema::create('ip_block', function (Blueprint $table) {
           $table->id('block_id');
           $table->unsignedInteger('site_id')->nullable()->index();
           $table->ipAddress('ip')->nullable()->index();
           $table->boolean('flag_proxy')->default(false);
           $table->string('reason', 200000)->nullable();
           $table->timestamp('date_blocked')->nullable();
        });

        Schema::create('license', function (Blueprint $table) {
           $table->id('license_id')->startingValue(16);
           $table->string('name', 100)->nullable()->unique();
           $table->string('description', 200000)->nullable();
           $table->unsignedInteger('sort')->default(0);
        });

        Schema::create('log_event', function (Blueprint $table) {
           $table->id('event_id');
           $table->timestamp('date')->nullable();
           $table->unsignedInteger('user_id')->nullable();
           $table->ipAddress('ip')->nullable();
           $table->ipAddress('proxy')->nullable();
           $table->string('type', 256)->nullable()->index();
           $table->unsignedInteger('site_id')->nullable()->index();
           $table->unsignedInteger('page_id')->nullable();
           $table->unsignedInteger('revision_id')->nullable();
           $table->unsignedInteger('thread_id')->nullable();
           $table->unsignedInteger('post_id')->nullable();
           $table->string('user_agent', 512)->nullable();
           $table->string('text', 200000)->nullable();
        });

        Schema::create('member', function (Blueprint $table) {
           $table->id('member_id');
           $table->unsignedInteger('site_id')->nullable();
           $table->unsignedInteger('user_id')->nullable();
           $table->timestamp('date_joined')->nullable();
           $table->boolean('allow_newsletter')->default(true);

           $table->unique(['site_id', 'user_id']);
        });

        Schema::create('member_application', function (Blueprint $table) {
           $table->id('application_id');
           $table->unsignedInteger('site_id')->nullable();
           $table->unsignedInteger('user_id')->nullable();
           $table->string('status', 20)->default('pending');
           $table->timestamp('date')->nullable();
           $table->string('comment', 200000);
           $table->string('reply', 200000);

           $table->unique(['site_id', 'user_id']);
        });

        Schema::create('member_invitation', function (Blueprint $table) {
           $table->id('invitation_id');
           $table->unsignedInteger('site_id')->nullable()->index();
           $table->unsignedInteger('user_id')->nullable()->index();
           $table->unsignedInteger('by_user_id')->nullable();
           $table->timestamp('date')->nullable();
           $table->string('body, 200000');
        });

        Schema::create('membership_link', function (Blueprint $table) {
           $table->id('link_id');
           $table->unsignedInteger('site_id')->nullable();
           $table->unsignedInteger('by_user_id')->nullable();
           $table->unsignedInteger('user_id')->nullable();
           $table->timestamp('date')->nullable();
           $table->string('type', 20)->nullable();
        });

        Schema::create('moderator', function (Blueprint $table) {
           $table->id('moderator_id');
           $table->unsignedInteger('site_id')->nullable();
           $table->unsignedInteger('user_id')->nullable();
           $table->string('permissions', 10)->nullable();

           $table->unique(['site_id', 'user_id']);
        });

        Schema::create('notification', function (Blueprint $table) {
           $table->id('notification_id');
           $table->unsignedInteger('user_id')->nullable()->index();
           $table->string('body', 200000)->nullable();
           $table->string('type', 50)->nullable();
           $table->boolean('viewed')->default(false);
           $table->timestamp('date')->nullable();
           $table->binary('extra')->nullable();
           $table->boolean('notify_online')->default(true);
           $table->boolean('notify_feed')->default(false);
           $table->boolean('notify_email')->default(true);
        });

        Schema::create('ozone_group', function (Blueprint $table) {
           $table->id('group_id');
           $table->unsignedInteger('parent_group_id')->nullable();
           $table->string('name', 50)->nullable()->unique();
           $table->string('description', 200000)->nullable();
        });

        Schema::create('ozone_group_permission_modifier', function (Blueprint $table) {
           $table->id('group_permission_id');
           $table->string('group_id', 20)->nullable();
           $table->string('permission_id', 20)->nullable();
           $table->integer('modifier')->nullable();
        });

        Schema::create('ozone_lock', function (Blueprint $table) {
           $table->string('key', 100)->primary();
        });

        Schema::create('ozone_permission', function (Blueprint $table) {
           $table->id('permission_id');
           $table->string('name', 50)->nullable()->unique();
           $table->string('description', 200000)->nullable();
        });

        Schema::create('ozone_session', function (Blueprint $table) {
            $table->string('session_id', 60)->primary();
            $table->timestamp('started')->nullable();
            $table->timestamp('last_accessed')->nullable();
            $table->string('ip_address', 90)->nullable();
            $table->boolean('check_ip')->default(false);
            $table->boolean('infinite')->default(false);
            $table->unsignedInteger('user_id')->nullable()->index();
            $table->binary('serialized_datablock')->nullable();
            $table->string('ip_address_ssl', 90)->nullable();
            $table->string('ua_hash', 256)->nullable();
        });

        Schema::create('ozone_user_group_relation', function (Blueprint $table) {
            $table->id('user_group_id');
            $table->unsignedInteger('user_id')->nullable();
            $table->unsignedInteger('group_id')->nullable();
        });

        Schema::create('ozone_user_permission_modifier', function (Blueprint $table) {
            $table->id('user_permission_id');
            $table->unsignedInteger('user_id')->nullable();
            $table->string('permission_id', 20)->nullable();
            $table->integer('modifier')->nullable();
        });

        Schema::create('page', function (Blueprint $table) {
            $table->id('page_id')->startingValue(53);
            $table->unsignedInteger('site_id')->nullable()->index();
            $table->unsignedInteger('category_id')->nullable()->index();
            $table->unsignedInteger('parent_page_id')->nullable()->index();
            $table->unsignedInteger('revision_id')->nullable()->index();
            $table->unsignedInteger('source_id')->nullable();
            $table->unsignedInteger('metadata_id')->nullable();
            $table->unsignedInteger('revision_number')->default(0);
            $table->string('title', 256)->nullable();
            $table->string('unix_name', 256)->nullable()->index();
            $table->timestamp('date_created')->nullable();
            $table->timestamp('date_last_edited')->nullable();
            $table->unsignedInteger('last_edit_user_id')->nullable();
            $table->string('last_edit_user_string', 80)->nullable();
            $table->unsignedInteger('thread_id')->nullable();
            $table->unsignedInteger('owner_user_id')->nullable();
            $table->boolean('blocked')->default(false);
            $table->integer('rate')->default(0);

            $table->unique(['site_id', 'unix_name']);
        });

        Schema::create('page_abuse_flag', function (Blueprint $table) {
            $table->id('flag_id');
            $table->unsignedInteger('user_id')->nullable();
            $table->unsignedInteger('site_id')->nullable()->index();
            $table->string('path', 100)->nullable();
            $table->boolean('site_valid')->default(true);
            $table->boolean('global_valid')->default(true);
        });

        Schema::create('page_compiled', function (Blueprint $table) {
            $table->unsignedInteger('page_id')->primary();
            $table->string('text', 200000)->nullable();
            $table->timestamp('date_compiled')->nullable();
        });

        Schema::create('page_edit_lock', function (Blueprint $table) {
            $table->id('lock_id');
            $table->unsignedInteger('page_id')->nullable()->index();
            $table->string('mode', 10)->default('page');
            $table->unsignedInteger('section_id')->nullable();
            $table->unsignedInteger('range_start')->nullable();
            $table->unsignedInteger('range_end')->nullable();
            $table->string('page_unix_name', 256)->nullable();
            $table->unsignedInteger('user_id')->nullable()->index();
            $table->string('user_string', 80)->nullable();
            $table->string('session_id', 60)->nullable();
            $table->timestamp('date_started')->nullable();
            $table->timestamp('date_last_accessed')->nullable();
            $table->string('secret', 100)->nullable();
            $table->unsignedInteger('site_id')->nullable();

            $table->unique(['site_id', 'page_unix_name']);
        });

        Schema::create('page_external_link', function (Blueprint $table) {
            $table->id('link_id');
            $table->unsignedInteger('site_id')->nullable();
            $table->unsignedInteger('page_id')->nullable();
            $table->string('to_url', 512)->nullable();
            $table->timestamp('date')->nullable();
        });

        Schema::create('page_inclusion', function (Blueprint $table) {
            $table->id('inclusion_id');
            $table->unsignedInteger('including_page_id')->nullable();
            $table->unsignedInteger('included_page_id')->nullable();
            $table->string('included_page_name', 128)->nullable();
            $table->unsignedInteger('site_id')->nullable()->index();

            $table->unique(['including_page_id', 'included_page_id', 'included_page_name']);
        });

        Schema::create('page_link', function (Blueprint $table) {
            $table->id('link_id')->startingValue(70);
            $table->unsignedInteger('from_page_id')->nullable();
            $table->unsignedInteger('to_page_id')->nullable();
            $table->string('to_page_name', 128)->nullable();
            $table->unsignedInteger('site_id')->nullable()->index();

            $table->unique(['from_page_id', 'to_page_id', 'to_page_name']);
        });

        Schema::create('page_metadata', function (Blueprint $table) {
            $table->id('metadata_id')->startingValue(57);
            $table->unsignedInteger('parent_page_id')->nullable();
            $table->string('title', 256)->nullable();
            $table->string('unix_name', 80)->nullable();
            $table->unsignedInteger('owner_user_id')->nullable();
        });

        Schema::create('page_rate_vote', function (Blueprint $table) {
            $table->id('rate_id');
            $table->unsignedInteger('user_id')->nullable();
            $table->unsignedInteger('page_id')->nullable();
            $table->integer('rate')->default(1);
            $table->timestamp('date')->nullable();

            $table->unique(['user_id', 'page_id']);
        });

        Schema::create('page_revision', function (Blueprint $table) {
            $table->id('revision_id')->startingValue(64);
            $table->unsignedInteger('page_id')->nullable()->index();
            $table->unsignedInteger('source_id')->nullable();
            $table->unsignedInteger('metadata_id')->nullable();
            $table->string('flags', 100)->nullable();
            $table->boolean('flag_text')->default(false);
            $table->boolean('flag_title')->default(false);
            $table->boolean('flag_file')->default(false);
            $table->boolean('flag_rename')->default(false);
            $table->boolean('flag_meta')->default(false);
            $table->boolean('flag_new')->default(false);
            $table->unsignedInteger('since_full_source')->nullable();
            $table->boolean('diff_source')->default(false);
            $table->unsignedInteger('revision_number')->nullable();
            $table->timestamp('date_last_edited')->nullable();
            $table->unsignedInteger('user_id')->nullable()->index();
            $table->string('user_string', 80)->nullable();
            $table->string('comments', 200000)->nullable();
            $table->boolean('flag_new_site')->default(false);
            $table->unsignedInteger('site_id')->nullable()->index();
        });

        Schema::create('page_source', function (Blueprint $table) {
            $table->id('source_id')->startingValue(63);
            $table->string('text', 200000)->nullable();
        });

        Schema::create('page_tag', function (Blueprint $table) {
            $table->id('tag_id');
            $table->unsignedInteger('site_id')->nullable()->index();
            $table->unsignedInteger('page_id')->nullable()->index();
            $table->string('tag', 20)->nullable();
        });

        Schema::create('private_message', function (Blueprint $table) {
            $table->id('message_id');
            $table->unsignedInteger('from_user_id')->nullable()->index();
            $table->unsignedInteger('to_user_id')->nullable()->index();
            $table->string('subject', 256)->nullable();
            $table->string('body', 200000)->nullable();
            $table->timestamp('date')->nullable();
            $table->unsignedInteger('flag')->nullable();
            $table->boolean('flag_new')->default(true);
        });

        Schema::create('private_user_block', function (Blueprint $table) {
            $table->id('block_id');
            $table->unsignedInteger('user_id')->nullable();
            $table->unsignedInteger('blocked_user_id')->nullable();

            $table->unique(['user_id', 'blocked_user_id']);
        });

        Schema::create('profile', function (Blueprint $table) {
            $table->unsignedInteger('user_id')->primary();
            $table->string('real_name', 70)->nullable();
            $table->string('pronouns', 30)->nullable();
            $table->unsignedInteger('birthday_day')->nullable();
            $table->unsignedInteger('birthday_month')->nullable();
            $table->unsignedInteger('birthday_year')->nullable();
            $table->string('about', 200000)->nullable();
            $table->string('location', 70)->nullable();
            $table->string('website', 100)->nullable();
            $table->string('im_icq', 100)->nullable();
            $table->string('im_jabber', 100)->nullable();
            $table->unsignedInteger('change_screen_name_count')->default(0);
        });

        Schema::create('site', function (Blueprint $table) {
            $table->id('site_id')->startingValue(4);
            $table->string('name', 50)->nullable();
            $table->string('subtitle', 60)->nullable();
            $table->string('unix_name', 80)->nullable()->unique();
            $table->string('description', 200000)->nullable();
            $table->string('language', 10)->default(env('DEFAULT_LANGUAGE', 'en'));
            $table->timestamp('date_created')->nullable();
            $table->string('custom_domain', 60)->nullable()->index();
            $table->boolean('visible')->default(true)->index();
            $table->string('default_page', 80)->default('start');
            $table->boolean('private')->default(false)->index();
            $table->boolean('deleted')->default(false);
        });

        Schema::create('site_backup', function (Blueprint $table) {
            $table->id('backup_id');
            $table->unsignedInteger('site_id')->nullable();
            $table->string('status', 50)->nullable();
            $table->boolean('backup_source')->default(true);
            $table->boolean('backup_files')->default(true);
            $table->timestamp('date')->nullable();
            $table->string('rand', 100)->nullable();
        });

        Schema::create('site_settings', function (Blueprint $table) {
            $table->unsignedInteger('site_id')->primary();
            $table->boolean('allow_membership_by_apply')->default(true);
            $table->boolean('allow_membership_by_password')->default(false);
            $table->string('membership_password', 80)->nullable();
            $table->unsignedInteger('file_storage_size')->default(314572800);
            $table->boolean('use_ganalytics')->default(false);
            $table->string('private_landing_page', 80)->default('system:join');
            $table->unsignedInteger('max_private_members')->default(50);
            $table->unsignedInteger('max_private_viewers')->default(20);
            $table->boolean('hide_navigation_unauthorized')->default(true);
            $table->string('ssl_mode', 20)->nullable();
            $table->boolean('allow_members_invite')->default(false);
            $table->unsignedInteger('max_upload_file_size')->default(10485760);
        });

        Schema::create('site_super_settings', function (Blueprint $table) {
            $table->unsignedInteger('site_id')->primary();
            $table->boolean('can_custom_domain')->default(true);
        });

        Schema::create('site_tag', function (Blueprint $table) {
            $table->id('tag_id')->startingValue(2);
            $table->unsignedInteger('site_id')->nullable();
            $table->string('tag', 20)->nullable();

            $table->unique(['site_id', 'tag']);
        });

        Schema::create('site_viewer', function (Blueprint $table) {
            $table->id('viewer_id');
            $table->unsignedInteger('site_id')->nullable();
            $table->unsignedInteger('user_id')->nullable();
        });

        Schema::create('theme', function (Blueprint $table) {
            $table->id('theme_id')->startingValue(29);
            $table->string('name', 100)->nullable();
            $table->string('unix_name', 100)->nullable();
            $table->boolean('abstract')->default(false);
            $table->unsignedInteger('extends_theme_id')->nullable();
            $table->unsignedInteger('variant_of_theme_id')->nullable();
            $table->boolean('custom')->default(false);
            $table->unsignedInteger('site_id')->nullable();
            $table->boolean('use_side_bar')->default(true);
            $table->boolean('use_top_bar')->default(true);
            $table->unsignedInteger('sort_index')->default(0);
            $table->string('sync_page_name', 100)->nullable();
            $table->unsignedInteger('revision_number')->default(0);
        });

        Schema::create('theme_preview', function (Blueprint $table) {
            $table->unsignedInteger('theme_id')->primary();
            $table->string('body', 200000)->nullable();
        });

        Schema::create('ucookie', function (Blueprint $table) {
            $table->string('ucookie_id', 100)->primary();
            $table->unsignedInteger('site_id')->nullable()->index();
            $table->string('session_id', 60)->nullable()->index();
            $table->timestamp('date_granted')->nullable();
        });

        Schema::create('user_abuse_flag', function (Blueprint $table) {
            $table->id('flag_id');
            $table->unsignedInteger('user_id')->nullable();
            $table->unsignedInteger('target_user_id')->nullable();
            $table->unsignedInteger('site_id')->nullable()->index();
            $table->boolean('site_valid')->default(true);
            $table->boolean('global_valid')->default(true);
        });

        Schema::create('user_block', function (Blueprint $table) {
            $table->id('block_id');
            $table->unsignedInteger('site_id')->nullable()->index();
            $table->unsignedInteger('user_id')->nullable();
            $table->string('reason', 200000)->nullable();
            $table->timestamp('date_blocked')->nullable();

            $table->unique(['site_id', 'user_id']);
        });

        Schema::create('user_karma', function (Blueprint $table) {
            $table->unsignedInteger('user_id')->nullable();
            $table->unsignedInteger('points')->nullable();
            $table->unsignedInteger('level')->nullable();
        });

        Schema::create('user_settings', function (Blueprint $table) {
            $table->unsignedInteger('user_id')->nullable();
            $table->boolean('receive_invitations')->default(true);
            $table->char('receive_pm', 5)->default('a');
            $table->string('notify_online', 512)->default('*');
            $table->string('notify_feed', 512)->default('*');
            $table->string('notify_email', 512)->nullable();
            $table->boolean('receive_newsletter')->default(true);
            $table->boolean('receive_digest')->default(true);
            $table->boolean('allow_site_newsletters_default')->default(true);
            $table->unsignedInteger('max_sites_admin')->default(3);
        });

        Schema::create('watched_forum_thread', function (Blueprint $table) {
            $table->id('watched_id');
            $table->unsignedInteger('user_id')->nullable();
            $table->unsignedInteger('thread_id')->nullable();

            $table->unique(['user_id', 'thread_id']);
        });

        Schema::create('watched_page', function (Blueprint $table) {
            $table->id('watched_id');
            $table->unsignedInteger('user_id')->nullable();
            $table->unsignedInteger('page_id')->nullable();

            $table->unique(['user_id', 'page_id']);
        });

        if(env('APP_ENV') == 'testing') {
            Artisan::call(
                'db:seed',
                [
                    '--class' => PHPUnitLegacySeeder::class,
                ]
            );
        }
        else {
            Artisan::call(
                'db:seed',
                [
                    '--class' => LegacySeeder::class,
                ]
            );
        }
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
        Schema::drop('admin');
        Schema::drop('admin_notification');
        Schema::drop('anonymous_abuse_flag');
        Schema::drop('category');
        Schema::drop('category_template');
        Schema::drop('comment');
        Schema::drop('comment_revision');
        Schema::drop('contact');
        Schema::drop('domain_redirect');
        Schema::drop('email_invitation');
        Schema::drop('file');
        Schema::drop('files_event');
        Schema::drop('forum_category');
        Schema::drop('forum_group');
        Schema::drop('forum_post');
        Schema::drop('forum_post_revision');
        Schema::drop('forum_settings');
        Schema::drop('forum_thread');
        Schema::drop('front_forum_feed');
        Schema::drop('global_ip_block');
        Schema::drop('global_user_block');
        Schema::drop('ip_block');
        Schema::drop('license');
        Schema::drop('log_event');
        Schema::drop('member');
        Schema::drop('member_application');
        Schema::drop('member_invitation');
        Schema::drop('membership_link');
        Schema::drop('moderator');
        Schema::drop('notification');
        Schema::drop('ozone_group');
        Schema::drop('ozone_group_permission_modifier');
        Schema::drop('ozone_lock');
        Schema::drop('ozone_permission');
        Schema::drop('ozone_session');
        Schema::drop('ozone_user_group_relation');
        Schema::drop('ozone_user_permission_modifier');
        Schema::drop('page');
        Schema::drop('page_abuse_flag');
        Schema::drop('page_compiled');
        Schema::drop('page_edit_lock');
        Schema::drop('page_external_link');
        Schema::drop('page_inclusion');
        Schema::drop('page_link');
        Schema::drop('page_metadata');
        Schema::drop('page_rate_vote');
        Schema::drop('page_revision');
        Schema::drop('page_source');
        Schema::drop('page_tag');
        Schema::drop('private_message');
        Schema::drop('private_user_block');
        Schema::drop('profile');
        Schema::drop('site');
        Schema::drop('site_backup');
        Schema::drop('site_settings');
        Schema::drop('site_super_settings');
        Schema::drop('site_tag');
        Schema::drop('site_viewer');
        Schema::drop('theme');
        Schema::drop('theme_preview');
        Schema::drop('ucookie');
        Schema::drop('user_abuse_flag');
        Schema::drop('user_block');
        Schema::drop('user_karma');
        Schema::drop('user_settings');
        Schema::drop('watched_forum_thread');
        Schema::drop('watched_page');
    }
}
