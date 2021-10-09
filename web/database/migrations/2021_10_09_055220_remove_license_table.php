<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class RemoveLicenseTable extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        // Drop old columns
        Schema::table('category', function (Blueprint $table) {
            $table->dropColumn('license_id');
            $table->dropColumn('license_other');
            $table->dropColumn('license_default');
        });

        // Add new columns
        Schema::table('category', function (Blueprint $table) {
            $table->string('license_id')->default('cc_by_sa_4');
            $table->boolean('license_default')->default(true);
        });

        // Remove license table
        Schema::drop('license');
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
        // Re-add license table
        Schema::create('license', function (Blueprint $table) {
            $table->id('license_id')->startingValue(16);
            $table->string('name', 100)->nullable()->unique();
            $table->string('description', 200000)->nullable();
            $table->unsignedInteger('sort')->default(0);
        });

        // Drop new columns
        Schema::table('category', function (Blueprint $table) {
            $table->dropColumn('license_id');
            $table->dropColumn('license_default');
        });

        // Re-add old columns
        Schema::table('category', function (Blueprint $table) {
            $table->boolean('license_default')->default(true);
            $table->unsignedInteger('license_id')->nullable();
            $table->string('license_other', 350)->nullable();
        });
    }
}
