<?php

namespace Database\Seeders;

use Exception;
use Illuminate\Database\Seeder;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Hash;
use Wikidot\Utils\WDStringUtils;

class UserSeeder extends Seeder
{
    /**
     * Run the database seeds.
     * Throws Exception if random_bytes() can't collect enough entropy.
     *
     * @return void
     * @throws Exception
     */
    public function run()
    {
        DB::table('users')->insert([
            [
                // ID 1
                'username' => 'admin',
                'unix_name' => WDStringUtils::toUnixName('admin'),
                'password' => Hash::make(env('ADMIN_INITIAL_PASSWORD', 'admin1')),
                'email' => 'admin@wikijump',
                'email_verified_at' => now(),
                'language' => env('DEFAULT_LANGUAGE'),
                'karma_points' => 110,
                'karma_level' => 2
            ],
            [
                // ID 2
                'username' => 'Automatic',
                'unix_name' => WDStringUtils::toUnixName('Automatic'),
                'password' => Hash::make(bin2hex(random_bytes(32))),
                'email' => 'automatic@wikijump',
                'email_verified_at' => now(),
                'language' => env('DEFAULT_LANGUAGE'),
            ],
            [
                // ID 3
                'username' => 'Anonymous',
                'unix_name' => WDStringUtils::toUnixName('Anonymous'),
                'password' => Hash::make(bin2hex(random_bytes(32))),
                'email' => 'anonymous@wikijump',
                'email_verified_at' => now(),
                'language' => env('DEFAULT_LANGUAGE'),
            ]
        ]);
        if(env('APP_ENV') != 'production') {
            DB::table('users')->insert(
                [
                    // ID 4 if not in prod
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
