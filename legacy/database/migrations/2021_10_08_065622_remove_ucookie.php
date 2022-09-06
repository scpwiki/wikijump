<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class RemoveUcookie extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        Schema::drop('ucookie');
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
        Schema::create('ucookie', function (Blueprint $table) {
            $table->string('ucookie_id', 100)->primary();
            $table->unsignedInteger('site_id')->nullable()->index();
            $table->string('session_id', 60)->nullable()->index();
            $table->timestamp('date_granted')->nullable();

            $table->foreign('session_id')
                ->references('session_id')->on('ozone_session')
                ->cascadeOnUpdate()->cascadeOnDelete();

            $table->foreign('site_id')
                ->references('site_id')->on('site')
                ->cascadeOnUpdate()->cascadeOnDelete();
        });
    }
}
