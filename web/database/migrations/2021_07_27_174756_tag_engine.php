<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class TagEngine extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {

        // Drop 'site_tag' table (entirely deprecated).
        Schema::drop('site_tag');

        // Create 'tag_settings' table.
        Schema::create('tag_settings', function (Blueprint $table) {
            $table->id('configuration_id')->autoIncrement();
            $table->unsignedInteger('site_id')->nullable()->index();
            $table->string('configuration_name')->default('Unnamed Configuration');
            $table->jsonb('allowed_tags')->default('[]');
        });

        // Add column to 'site' that enables/disables the tag engine.
        Schema::table('site', function (Blueprint $table) {
            $table->boolean('enable_tag_engine')->default(false);
        });
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {

        // Recreate 'site_tag' from legacy tables.
        Schema::create('site_tag', function (Blueprint $table) {
            $table->id('tag_id')->startingValue(2);
            $table->unsignedInteger('site_id')->nullable();
            $table->string('tag', 20)->nullable();
            $table->unique(['site_id', 'tag']);
        });

        // Drop new 'tag_settings' table.
        Schema::drop('tag_settings');

        // Drop new 'enable_tag_engine' column in 'site' table.
        Schema::table('site', function (Blueprint $table) {
            $table->dropColumn('enable_tag_engine');
        });
    }
}
