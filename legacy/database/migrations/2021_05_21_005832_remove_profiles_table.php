<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class RemoveProfilesTable extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        Schema::drop('profile');
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
        Schema::create('profile', function (Blueprint $table) {
            $table->unsignedInteger('user_id')->primary();
            $table->string('real_name', 70)->nullable();
            $table->string('pronouns', 30)->nullable();
            $table->unsignedInteger('birthday_day')->nullable();
            $table->unsignedInteger('birthday_month')->nullable();
            $table->unsignedInteger('birthday_year')->nullable();
            $table->string('about', 200000)->nullable();
            $table->string('location', 70)->nullable();
            $table->string('website', 100)->nullable();
            $table->string('im_icq', 100)->nullable();
            $table->string('im_jabber', 100)->nullable();
            $table->unsignedInteger('change_screen_name_count')->default(0);
        });
    }
}
