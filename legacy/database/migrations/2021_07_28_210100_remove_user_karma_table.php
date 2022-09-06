<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class RemoveUserKarmaTable extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        Schema::drop('user_karma');
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
        Schema::create('user_karma', function (Blueprint $table) {
            $table->unsignedInteger('user_id')->nullable();
            $table->unsignedInteger('points')->nullable();
            $table->unsignedInteger('level')->nullable();
        });
    }
}
