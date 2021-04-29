<?php

namespace Database\Seeders;

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
        DB::table('users')->insert(
            [
                'username' => 'admin',
                'unix_name' => WDStringUtils::toUnixName('admin'),
                'password' => env('ADMIN_INITIAL_PASSWORD', Hash::make('admin1')),
                'email' => 'admin@wikijump',
                'email_verified_at' => now(),
                'language' => env('DEFAULT_LANGUAGE'),
            ]
        );
        if(env('APP_ENV') != 'production') {
            DB::table('users')->insert(
                [
                    'username' => 'user',
                    'unix_name' => WDStringUtils::toUnixName('user'),
                    'password' => Hash::make('user1'),
                    'email' => 'user@wikijump.dev',
                    'email_verified_at' => now(),
                    'language' => env('DEFAULT_LANGUAGE'),
                ]
            );
        }
    }
}
