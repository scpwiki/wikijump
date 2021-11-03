<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class ModifyTagsTable extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        // Creating new column in Page table.
        Schema::table('page', function (Blueprint $table) {
            $table->jsonb('tags')->default('[]');
        });

        // Moving all tags from 'page_tag' table to new column in 'page' table.
        $page_ids = DB::table('page_tag')->pluck('page_id')->toArray();

        foreach ($page_ids as $page_id) {
            $tags = DB::table('page_tag')->where('page_id', $page_id)->pluck('tag')->toArray();

            DB::table('page')
                ->where('page_id', $page_id)
                ->update(['tags' => $tags]);
        }

        // Drop 'page_tag' table.
        Schema::drop('page_tag');
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
        // Recreates page_tag table.
        Schema::create('page_tag', function (Blueprint $table) {
            $table->id('tag_id');
            $table->unsignedInteger('site_id')->nullable()->index();
            $table->unsignedInteger('page_id')->nullable()->index();
            $table->string('tag', 20)->nullable();
        });

        // Remove column from page table.
        Schema::table('page', function (Blueprint $table) {
            $table->dropColumn('tags');
        });

    }
}
