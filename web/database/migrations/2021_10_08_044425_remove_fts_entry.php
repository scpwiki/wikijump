<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class RemoveFtsEntry extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        Schema::drop('fts_entry');
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
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

        /**
         * SQLite has no concept of a TSVector so we can't add it or seed the data.
         */
        if(env('APP_ENV') !== 'testing') {
            DB::statement('ALTER TABLE fts_entry ADD COLUMN vector TSVECTOR');

            Artisan::call(
                'db:seed',
                [
                    '--class' => FtsSeeder::class,
                ]
            );
        }
    }
}
