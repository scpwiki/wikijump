<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class RemoveSiteBackup extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        Schema::drop('site_backup');
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
        Schema::create('site_backup', function (Blueprint $table) {
            $table->id('backup_id');
            $table->unsignedInteger('site_id')->nullable();
            $table->string('status', 50)->nullable();
            $table->boolean('backup_source')->default(true);
            $table->boolean('backup_files')->default(true);
            $table->timestamp('date')->nullable();
            $table->string('rand', 100)->nullable();
        });
    }
}
