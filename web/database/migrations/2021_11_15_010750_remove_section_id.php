<?php
declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class RemoveSectionId extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        Schema::table('page_edit_lock', function (Blueprint $table) {
            $table->dropColumn('section_id');
            $table->dropColumn('range_start');
            $table->dropColumn('range_end');
        });
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
        Schema::table('page_edit_lock', function (Blueprint $table) {
            $table->unsignedInteger('section_id')->nullable();
            $table->unsignedInteger('range_start')->nullable();
            $table->unsignedInteger('range_end')->nullable();
        });
    }
}
