<?php
declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class LegacyCompatibilityIds extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        // Temporary compatibility IDs while these tables are still in Ozone
        DB::statement('ALTER SEQUENCE forum_post_post_id_seq         RESTART WITH  7000000');
        DB::statement('ALTER SEQUENCE forum_thread_thread_id_seq     RESTART WITH 30000000');
        DB::statement('ALTER SEQUENCE forum_category_category_id_seq RESTART WITH  9000000');
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
        DB::statement('ALTER SEQUENCE forum_post_post_id_seq         RESTART WITH 1');
        DB::statement('ALTER SEQUENCE forum_thread_thread_id_seq     RESTART WITH 1');
        DB::statement('ALTER SEQUENCE forum_category_category_id_seq RESTART WITH 1');
    }
}
