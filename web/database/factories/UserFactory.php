<?php

namespace Database\Factories;

use Illuminate\Support\Facades\Hash;
use Wikidot\Utils\WDStringUtils;
use Wikijump\Models\User;
use Illuminate\Database\Eloquent\Factories\Factory;
use Illuminate\Support\Str;

/**
 * Specification for creating users.
 * @package Database\Factories
 */
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
    public function definition(): array
    {
        $username = $this->faker->unique()->userName;
        return [
            'username' => $username,
            'unix_name' => WDStringUtils::toUnixName($username),
            'email' => $this->faker->unique()->freeEmail,
            'email_verified_at' => now(),
            'password' => Hash::make('password'),
            'remember_token' => Str::random(10),
            'language' => env('DEFAULT_LANGUAGE', 'en'),
            'real_name' => $this->faker->name,
            'dob' => $this->faker
                ->dateTimeBetween('1950-01-01', '2006-12-31')
                ->format('Y-m-d'),
            'bio' => $this->faker->realText('2000'),
            'about_page' => 'https://'.$this->faker->domainName.'/'.$this->faker->domainWord
        ];
    }

    /**
     * Indicate that the model's email address should be unverified.
     *
     * @return Factory
     */
    public function unverified() : Factory
    {
        return $this->state(function (array $attributes) {
            return [
                'email_verified_at' => null,
            ];
        });
    }
}
