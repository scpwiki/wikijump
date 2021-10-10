<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class RemoveLogEvent extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        Schema::drop('log_event');
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
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
    }
}
