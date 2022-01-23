<?php
declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class Jobs extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        // Creates a temporary DEEPWELL table
        // This is used to store asynchronous jobs.

        DB::statement("
            CREATE TABLE job (
                job_id SERIAL PRIMARY KEY,
                job_type TEXT NOT NULL,
                job_data JSON NOT NULL
            )
        ");
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
        DB::statement('DROP TABLE job');
    }
}
