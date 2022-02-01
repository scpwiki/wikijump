<?php
declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class DeletePageEditLock extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        Schema::drop('page_edit_lock');
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
        Schema::create('page_edit_lock', function (Blueprint $table) {
            $table->id('lock_id');
            $table->unsignedInteger('page_id')->nullable()->index();
            $table->string('mode', 10)->default('page');
            $table->unsignedInteger('section_id')->nullable();
            $table->unsignedInteger('range_start')->nullable();
            $table->unsignedInteger('range_end')->nullable();
            $table->string('page_unix_name', 256)->nullable();
            $table->unsignedInteger('user_id')->nullable()->index();
            $table->string('user_string', 80)->nullable();
            $table->string('session_id', 60)->nullable();
            $table->timestamp('date_started')->nullable();
            $table->timestamp('date_last_accessed')->nullable();
            $table->string('secret', 100)->nullable();
            $table->unsignedInteger('site_id')->nullable();

            $table->unique(['site_id', 'page_unix_name']);
        });
    }
}
