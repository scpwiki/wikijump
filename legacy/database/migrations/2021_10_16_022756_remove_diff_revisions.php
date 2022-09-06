<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class RemoveDiffRevisions extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        Schema::table('page_revision', function (Blueprint $table) {
            $table->dropColumn('since_full_source');
            $table->dropColumn('diff_source');
            $table->dropColumn('flag_new_site');
        });
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
        Schema::table('page_revision', function (Blueprint $table) {
            $table->unsignedInteger('since_full_source')->nullable();
            $table->boolean('diff_source')->default(false);
            $table->boolean('flag_new_site')->default(false);
        });
    }
}
