<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class DropContactTable extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        Schema::drop('contact');
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
        Schema::create('contact', function (Blueprint $table) {
            $table->id('contact_id');
            $table->unsignedInteger('user_id')->nullable();
            $table->unsignedInteger('target_user_id')->nullable();

            $table->unique(['user_id', 'target_user_id']);
        });
    }
}
