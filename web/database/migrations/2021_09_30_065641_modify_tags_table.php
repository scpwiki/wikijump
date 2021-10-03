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
      $pages = DB::table('page_tag')->pluck('page_id')->toArray();
      foreach ($pages as $page) {
        $tags = DB::table('page_tag')->where('page_id', $page)->pluck('tag')->toArray();

        DB::table('page')
          ->where('page_id', $page)
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
        //
    }
}
