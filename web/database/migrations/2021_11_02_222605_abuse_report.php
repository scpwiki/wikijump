<?php
declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class AbuseReport extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        Schema::drop('anonymous_abuse_flag');
        Schema::drop('page_abuse_flag');
        Schema::drop('user_abuse_flag');

        Schema::create('abuse_report', function(Blueprint $table) {
            $table->id();
            $table->timestamps();
            $table->foreignId('user_id');
            $table->foreignId('entity_id');
            $table->enum('entity_type', ['page', 'user']);
            $table->string('reason');

            $table->unique(['user_id', 'entity_id', 'entity_type']);
            $table->foreign('user_id')->references('id')->on('users');
        });
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
        Schema::drop('abuse_report');

        Schema::create('anonymous_abuse_flag', function(Blueprint $table) {
            $table->id('flag_id');
            $table->unsignedInteger('user_id')->nullable();
            $table->ipAddress('address')->nullable()->index();
            $table->boolean('proxy')->default(false)->nullable();
            $table->unsignedInteger('site_id')->nullable()->index();
            $table->boolean('site_valid')->default(true)->nullable();
            $table->boolean('global_valid')->default(true)->nullable();
        });

        Schema::create('page_abuse_flag', function (Blueprint $table) {
            $table->id('flag_id');
            $table->unsignedInteger('user_id')->nullable();
            $table->unsignedInteger('site_id')->nullable()->index();
            $table->string('path', 100)->nullable();
            $table->boolean('site_valid')->default(true);
            $table->boolean('global_valid')->default(true);
        });

        Schema::create('user_abuse_flag', function (Blueprint $table) {
            $table->id('flag_id');
            $table->unsignedInteger('user_id')->nullable();
            $table->unsignedInteger('target_user_id')->nullable();
            $table->unsignedInteger('site_id')->nullable()->index();
            $table->boolean('site_valid')->default(true);
            $table->boolean('global_valid')->default(true);
        });
    }
}
