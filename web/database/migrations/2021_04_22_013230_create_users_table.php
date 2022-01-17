<?php
declare(strict_types=1);

use Database\Seeders\UserSeeder;
use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Artisan;
use Illuminate\Support\Facades\Schema;

/**
 * Build users table (supports Fortify/Sanctum/Jetstream).
 */
class CreateUsersTable extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        Schema::create('users', function (Blueprint $table) {
            $table->id();
            $table->string('username')->unique();
            $table->string('unix_name')->unique();
            $table->unsignedSmallInteger('username_changes')->default(0);
            $table->string('email')->unique();
            $table->timestamp('email_verified_at')->nullable();
            $table->string('password');
            $table->text('two_factor_secret')->nullable();
            $table->text('two_factor_recovery_codes')->nullable();
            $table->rememberToken();
            $table->string('language',10)->nullable();
            $table->unsignedMediumInteger('karma_points')->default(0);
            $table->unsignedTinyInteger('karma_level')->default(0);
            $table->string('real_name', 30)->nullable();
            $table->string('pronouns', 30)->nullable();
            $table->date('dob')->nullable();
            $table->string('bio', 2000)->nullable();
            $table->string('about_page', 80)->nullable();
            $table->string('profile_photo_path', 2048)->nullable();
            $table->timestamps();
            $table->softDeletes();
        });

        Artisan::call('db:seed', [
            '--class' => UserSeeder::class,
        ]);
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
        Schema::dropIfExists('users');
    }
}
