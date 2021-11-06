<?php
declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class RemoveAbuseTables extends Migration
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
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
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
