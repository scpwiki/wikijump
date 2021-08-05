<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class DropPrivateUserBlockTable extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        Schema::drop('private_user_block');
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
        Schema::create('private_user_block', function (Blueprint $table) {
            $table->id('block_id');
            $table->unsignedInteger('user_id')->nullable();
            $table->unsignedInteger('blocked_user_id')->nullable();

            $table->unique(['user_id', 'blocked_user_id']);
        });
    }
}
