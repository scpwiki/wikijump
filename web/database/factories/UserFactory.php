<?php

namespace Database\Factories;

use Illuminate\Support\Facades\Hash;
use Wikidot\Utils\WDStringUtils;
use Wikijump\Models\User;
use Illuminate\Database\Eloquent\Factories\Factory;
use Illuminate\Support\Str;

class UserFactory extends Factory
{
    /**
     * The name of the factory's corresponding model.
     *
     * @var string
     */
    protected $model = User::class;

    /**
     * Define the model's default state.
     *
     * @return array
     */
    public function definition()
    {
        $username = $this->faker->userName;
        return [
            'username' => $username,
            'unix_name' => WDStringUtils::toUnixName($username),
            'email' => $this->faker->unique()->freeEmail,
            'email_verified_at' => now(),
            'password' => Hash::make('password'),
            'remember_token' => Str::random(10),
            'language' => env('DEFAULT_LANGUAGE')
        ];
    }
}
