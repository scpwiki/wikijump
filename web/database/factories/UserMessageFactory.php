<?php
declare(strict_types=1);

namespace Database\Factories;

use Illuminate\Database\Eloquent\Factories\Factory;
use Wikijump\Models\UserMessage;

/**
 * Factory for user-to-user private messages.
 * @package Database\Factories
 */
class UserMessageFactory extends Factory
{
    /**
     * The name of the factory's corresponding model.
     *
     * @var string
     */
    protected $model = UserMessage::class;

    /**
     * Define the model's default state.
     *
     * @return array
     */
    public function definition(): array
    {
        return [
            'subject' => $this->faker->sentence,
            'body' => $this->faker->paragraph,
            'flags' => 0 // Unread, non-draft, unstarred, unarchived message.
        ];
    }
}
