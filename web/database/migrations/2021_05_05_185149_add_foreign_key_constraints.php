<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class AddForeignKeyConstraints extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        Schema::table('admin', function(Blueprint $table) {
            $table->foreign('site_id')
                ->references('site_id')->on('site')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('admin_notification', function(Blueprint $table) {
            $table->foreign('site_id')
                ->references('site_id')->on('site')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('category', function (Blueprint $table) {
            $table->foreign('site_id')
                ->references('site_id')->on('site')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('domain_redirect', function (Blueprint $table) {
            $table->foreign('site_id')
                ->references('site_id')->on('site')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('email_invitation', function(Blueprint $table) {
            $table->foreign('site_id')
                ->references('site_id')->on('site')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('file', function (Blueprint $table) {
            $table->foreign('page_id')
                ->references('page_id')->on('page')
                ->cascadeOnUpdate()->cascadeOnDelete();

            $table->foreign('site_id')
                ->references('site_id')->on('site')
                ->cascadeOnUpdate()->nullOnDelete();
        });

        Schema::table('forum_category', function (Blueprint $table) {
            $table->foreign('group_id')
                ->references('group_id')->on('forum_group')
                ->cascadeOnUpdate()->restrictOnDelete();

            $table->foreign('last_post_id')
                ->references('post_id')->on('forum_post')
                ->cascadeOnUpdate()->nullOnDelete();

            $table->foreign('site_id')
                ->references('site_id')->on('site')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('forum_group', function (Blueprint $table) {
            $table->foreign('site_id')
                ->references('site_id')->on('site')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('forum_post', function (Blueprint $table) {
            $table->foreign('site_id')
                ->references('site_id')->on('site')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('forum_settings', function (Blueprint $table) {
            $table->foreign('site_id')
                ->references('site_id')->on('site')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('forum_thread', function (Blueprint $table) {
            $table->foreign('site_id')
                ->references('site_id')->on('site')
                ->cascadeOnUpdate()->cascadeOnDelete();

            $table->foreign('category_id')
                ->references('category_id')->on('forum_category')
                ->cascadeOnUpdate()->cascadeOnDelete();

            $table->foreign('page_id')
                ->references('page_id')->on('page')
                ->cascadeOnUpdate()->cascadeOnDelete();

            $table->foreign('last_post_id')
                ->references('post_id')->on('forum_post')
                ->cascadeOnUpdate()->nullOnDelete();
        });

        Schema::table('front_forum_feed', function (Blueprint $table) {
            $table->foreign('page_id')
                ->references('page_id')->on('page')
                ->cascadeOnUpdate()->cascadeOnDelete();

            $table->foreign('site_id')
                ->references('site_id')->on('site')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('ip_block', function (Blueprint $table) {
            $table->foreign('site_id')
                ->references('site_id')->on('site')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('log_event', function (Blueprint $table) {
            $table->foreign('site_id')
                ->references('site_id')->on('site')
                ->cascadeOnUpdate()->nullOnDelete();
        });

        Schema::table('member', function (Blueprint $table) {
            $table->foreign('site_id')
                ->references('site_id')->on('site')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('member_application', function (Blueprint $table) {
            $table->foreign('site_id')
                ->references('site_id')->on('site')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('member_invitation', function (Blueprint $table) {
            $table->foreign('site_id')
                ->references('site_id')->on('site')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('moderator', function (Blueprint $table) {
             $table->foreign('site_id')
                ->references('site_id')->on('site')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('page', function (Blueprint $table) {
            $table->foreign('parent_page_id')
                ->references('page_id')->on('page')
                ->cascadeOnUpdate()->nullOnDelete();

            $table->foreign('site_id')
                ->references('site_id')->on('site')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('page_abuse_flag', function (Blueprint $table) {
            $table->foreign('site_id')
                ->references('site_id')->on('site')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('page_compiled', function (Blueprint $table) {
            $table->foreign('page_id')
                ->references('page_id')->on('page')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('page_edit_lock', function (Blueprint $table) {
            $table->foreign('page_id')
                ->references('page_id')->on('page')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('page_inclusion', function (Blueprint $table) {
            $table->foreign('included_page_id')
                ->references('page_id')->on('page')
                ->cascadeOnUpdate()->cascadeOnDelete();

            $table->foreign('including_page_id')
                ->references('page_id')->on('page')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('page_link', function (Blueprint $table) {
            $table->foreign('from_page_id')
                ->references('page_id')->on('page')
                ->cascadeOnUpdate()->cascadeOnDelete();

            $table->foreign('to_page_id')
                ->references('page_id')->on('page')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('page_rate_vote', function (Blueprint $table) {
            $table->foreign('page_id')
                ->references('page_id')->on('page')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('page_tag', function (Blueprint $table) {
            $table->foreign('page_id')
                ->references('page_id')->on('page')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('site_backup', function (Blueprint $table) {
            $table->foreign('site_id')
                ->references('site_id')->on('site')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('site_settings', function (Blueprint $table) {
            $table->foreign('site_id')
                ->references('site_id')->on('site')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('site_super_settings', function (Blueprint $table) {
            $table->foreign('site_id')
                ->references('site_id')->on('site')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('site_tag', function (Blueprint $table) {
            $table->foreign('site_id')
                ->references('site_id')->on('site')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('site_viewer', function (Blueprint $table) {
            $table->foreign('site_id')
                ->references('site_id')->on('site')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('theme', function (Blueprint $table) {
            $table->foreign('site_id')
                ->references('site_id')->on('site')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('ucookie', function (Blueprint $table) {
            $table->foreign('session_id')
                ->references('session_id')->on('ozone_session')
                ->cascadeOnUpdate()->cascadeOnDelete();

            $table->foreign('site_id')
                ->references('site_id')->on('site')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('user_abuse_flag', function (Blueprint $table) {
            $table->foreign('site_id')
                ->references('site_id')->on('site')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('user_block', function (Blueprint $table) {
            $table->foreign('site_id')
                ->references('site_id')->on('site')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('watched_forum_thread', function (Blueprint $table) {
            $table->foreign('thread_id')
                ->references('thread_id')->on('forum_thread')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });

        Schema::table('watched_page', function (Blueprint $table) {
            $table->foreign('page_id')
                ->references('page_id')->on('page')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
        //
    }
}
