<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class NewPageLinks extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        Schema::drop('page_external_link');
        Schema::drop('page_inclusion');
        Schema::drop('page_link');

        // We don't use timestamps() because we don't need updated_at.

        Schema::create('page_link', function (Blueprint $table) {
            $table->id();
            $table->timestamp('created_at')->useCurrent();
            $table->foreignId('page_id');
            $table->foreignId('site_id');
            $table->string('url');
            $table->unsignedSmallInteger('count');

            $table->unique(['page_id', 'site_id', 'url']);
        });

        Schema::create('page_connection', function (Blueprint $table) {
            $table->id();
            $table->timestamp('created_at')->useCurrent();
            $table->foreignId('from_page_id');
            $table->foreignId('from_site_id');
            $table->foreignId('to_page_id');
            $table->foreignId('to_site_id');
            $table->unsignedSmallInteger('count');
            $table->set('connection_type', ['include-messy', 'include-elements', 'component', 'link']);

            $table->unique(['from_page_id', 'from_site_id', 'to_page_id', 'to_site_id', 'connection_type']);
        });

        Schema::create('page_connection_missing', function (Blueprint $table) {
            $table->id();
            $table->timestamp('created_at')->useCurrent();
            $table->foreignId('from_page_id');
            $table->foreignId('from_site_id');
            $table->string('to_page_name');
            $table->string('to_site_name')->nullable();
            $table->unsignedSmallInteger('count');
            $table->set('connection_type', ['include-messy', 'include-elements', 'component', 'link']);

            $table->unique(['from_page_id', 'from_site_id', 'to_page_name', 'to_site_name', 'count']);
        });
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
        Schema::drop('page_link');
        Schema::drop('page_connection');
        Schema::drop('page_connection_missing');

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
    }
}
