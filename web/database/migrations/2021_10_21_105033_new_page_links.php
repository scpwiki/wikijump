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

        Schema::create('page_link', function (Blueprint $table) {
            $table->id();
            $table->timestamp('created_at');
            $table->foreignId('page_id');
            $table->foreignId('site_id');
            $table->string('url');
            $table->unsignedSmallInteger('count');

            $table->unique(['page_id', 'site_id', 'url']);
        });

        Schema::create('page_connection', function (Blueprint $table) {
            $table->id();
            $table->timestamp('created_at');
            $table->foreignId('from_page_id');
            $table->foreignId('from_site_id');
            $table->foreignId('to_page_id');
            $table->foreignId('to_site_id');
            $table->set('connection_type', ['include-messy', 'include-elements', 'component', 'link']);

            $table->unique(['from_page_id', 'from_site_id', 'to_page_id', 'to_site_id', 'connection_type']);
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
