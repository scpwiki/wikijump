<?php

namespace Database\Seeders;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Seeder;

class DatabaseSeeder extends Seeder
{
    /**
     * Seed the application's database.
     *
     * @return void
     */
    public function run()
    {
        /**
         * Normally we'd call our seeder classes here. However, due to the
         * idempotent nature of the deployment, it's necessary to ensure that
         * seeding happens only once, and the php container runs it.
         *
         * To facilitate that, the seed calls are actually made within the
         * migration files. This way if the migration has ran, the seeder will
         * not run again.
         */
    }
}
