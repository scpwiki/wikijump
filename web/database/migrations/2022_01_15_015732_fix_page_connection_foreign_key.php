<?php
declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class FixPageConnectionForeignKey extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        DB::statement('ALTER TABLE page_connection_missing DROP CONSTRAINT page_connection_missing_to_site_id_fkey');
        DB::statement('ALTER TABLE page_connection_missing ADD FOREIGN KEY to_site_id REFERENCES site(site_id)');
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
        DB::statement('ALTER TABLE page_connection_missing DROP CONSTRAINT page_connection_missing_to_site_id_fkey');
        DB::statement('ALTER TABLE page_connection_missing ADD FOREIGN KEY to_site_id REFERENCES page(page_id)');
    }
}
