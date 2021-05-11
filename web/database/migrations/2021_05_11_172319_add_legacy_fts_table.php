<?php

use Database\Seeders\FtsSeeder;
use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Artisan;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Schema;

class AddLegacyFtsTable extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        Schema::create('fts_entry', function (Blueprint $table) {
            $table->id('fts_id')->startingValue(64);
            $table->integer('page_id')->nullable();
            $table->string('title', 256)->nullable();
            $table->string('unix_name', 100)->nullable();
            $table->unsignedInteger('thread_id')->nullable();
            $table->unsignedInteger('site_id')->nullable();
            $table->text('text')->nullable();
        });
        DB::statement('ALTER TABLE fts_entry ADD COLUMN vector TSVECTOR');

        Artisan::call('db:seed', [
            '--class' => FtsSeeder::class,
        ]);
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
        Schema::drop('fts_entry');
    }
}
