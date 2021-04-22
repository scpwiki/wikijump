<?php

namespace Database\Seeders;

use Carbon\Carbon;
use Illuminate\Database\Seeder;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Hash;
use Wikidot\Utils\WDStringUtils;

class UserSeeder extends Seeder
{
    /**
     * Run the database seeds.
     *
     * @return void
     */
    public function run()
    {
        /*
         * TODO: It would be good if production credentials were either
         * generated on the fly or if the admin were prompted for input.
         */
        if(env('APP_ENV') != 'production') {
            DB::table('users')->insert(
                [
                    'username' => 'admin',
                    'unix_name' => WDStringUtils::toUnixName('admin'),
                    'password' => Hash::make('admin1'),
                    'email' => 'admin@wikijump.dev',
                    'email_verified_at' => Carbon::now(),
                    'language' => env('DEFAULT_LANGUAGE'),
                ]
            );
            DB::table('users')->insert(
                [
                    'username' => 'user',
                    'unix_name' => WDStringUtils::toUnixName('user'),
                    'password' => Hash::make('user1'),
                    'email' => 'user@wikijump.dev',
                    'email_verified_at' => Carbon::now(),
                    'language' => env('DEFAULT_LANGUAGE'),
                ]
            );
        }
    }
}
