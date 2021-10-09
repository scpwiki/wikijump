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
        Schema::table('category', function (Blueprint $table) {
            $table->dropColumn('license_id');
            $table->dropColumn('license_other');

            $table->string('license_id');
            $table->renameColumn('license_default', 'license_inherits');
        });

        Schema::drop('license');
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
        Schema::create('license', function (Blueprint $table) {
            $table->id('license_id')->startingValue(16);
            $table->string('name', 100)->nullable()->unique();
            $table->string('description', 200000)->nullable();
            $table->unsignedInteger('sort')->default(0);
        });

        Schema::table('category', function (Blueprint $table) {
            $table->dropColumn('license_id');
            $table->renameColumn('license_inherits', 'license_default');

            $table->unsignedInteger('license_id')->nullable();
            $table->string('license_other', 350)->nullable();
        });
    }
}
