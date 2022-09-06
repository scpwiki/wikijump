<?php
declare(strict_types=1);

namespace Database\Seeders;

use Exception;
use Illuminate\Database\Seeder;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Hash;
use Wikidot\Utils\WDStringUtils;

/**
 * Seeder of `users` table.
 * @package Database\Seeders
 */
class UserSeeder extends Seeder
{
    /**
     * Run the database seeds.
     * Throws Exception if random_bytes() can't collect enough entropy.
     *
     * @return void
     * @throws Exception
     */
    public function run(): void
    {
        DB::table('users')->insert([
            [
                'id' => 1,
                'username' => 'Administrator',
                'unix_name' => WDStringUtils::toUnixName('admin'),
                'password' => Hash::make(env('ADMIN_INITIAL_PASSWORD', 'admin1')),
                'email' => 'admin@wikijump',
                'email_verified_at' => now(),
                'language' => env('DEFAULT_LANGUAGE'),
                'karma_points' => 110,
                'karma_level' => 2,
            ],
            [
                'id' => 2,
                'username' => 'Automatic',
                'unix_name' => WDStringUtils::toUnixName('Automatic'),
                'password' => Hash::make(bin2hex(random_bytes(32))),
                'email' => 'automatic@wikijump',
                'email_verified_at' => now(),
                'language' => env('DEFAULT_LANGUAGE'),
                'karma_points' => 0,
                'karma_level' => 0,
            ],
            [
                'id' => 3,
                'username' => 'Anonymous',
                'unix_name' => WDStringUtils::toUnixName('Anonymous'),
                'password' => Hash::make(bin2hex(random_bytes(32))),
                'email' => 'anonymous@wikijump',
                'email_verified_at' => now(),
                'language' => env('DEFAULT_LANGUAGE'),
                'karma_points' => 0,
                'karma_level' => 0,
            ],
        ]);
        if (env('APP_ENV') !== 'production') {
            // Sample user if not in prod
            DB::table('users')->insert([
                'id' => 4,
                'username' => 'User',
                'unix_name' => WDStringUtils::toUnixName('user'),
                'password' => Hash::make('user1'),
                'email' => 'user@wikijump.dev',
                'email_verified_at' => now(),
                'language' => env('DEFAULT_LANGUAGE'),
                'karma_points' => 0,
                'karma_level' => 0,
            ]);
        }
    }
}
