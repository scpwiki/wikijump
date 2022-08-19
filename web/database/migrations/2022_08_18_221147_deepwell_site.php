<?php
declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class DeepwellSite extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {

        DB::statement("ALTER TABLE site ALTER COLUMN name TYPE text");
        DB::statement("ALTER TABLE site ALTER COLUMN name SET NOT NULL");
        DB::statement("ALTER TABLE site ALTER COLUMN subtitle TYPE text");
        DB::statement("ALTER TABLE site ALTER COLUMN subtitle SET NOT NULL");
        DB::statement("ALTER TABLE site ALTER COLUMN description TYPE TEXT");
        DB::statement("ALTER TABLE site ALTER COLUMN description SET NOT NULL");
        DB::statement("ALTER TABLE site ALTER COLUMN date_created TYPE TIMESTAMP WITH TIME ZONE");
        DB::statement("ALTER TABLE site ALTER COLUMN date_created SET NOT NULL");
        DB::statement("ALTER TABLE site RENAME COLUMN language TO locale");
        DB::statement("ALTER TABLE site RENAME COLUMN date_created TO created_at");
        DB::statement("ALTER TABLE site ADD COLUMN updated_at TIMESTAMP WITH TIME ZONE");
        DB::statement("ALTER TABLE site ADD COLUMN deleted_at TIMESTAMP WITH TIME ZONE");
        DB::statement("ALTER TABLE site DROP COLUMN visible");
        DB::statement("ALTER TABLE site DROP COLUMN private");
        DB::statement("ALTER TABLE site DROP COLUMN deleted");

        Schema::drop('site_viewer');
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
        /* TODO */
    }
}
