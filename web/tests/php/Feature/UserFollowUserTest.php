<?php
declare(strict_types=1);

namespace Tests\Feature;

use Illuminate\Foundation\Testing\RefreshDatabase;
use Tests\TestCase;
use Wikijump\Models\User;

/**
 * Class UserFollowUserTest
 * A feature test for User methods invoking Interactions.
 * @package Tests\Feature
 */
class UserFollowUserTest extends TestCase
{
    use RefreshDatabase;

    private User $user;
    private User $user_to_follow;

    protected function setUp(): void
    {
        parent::setUp();
        $user = User::factory()->make();
        $user->save();
        $user_to_follow = User::factory()->make();
        $user_to_follow->save();

        $this->user = $user;
        $this->user_to_follow = $user_to_follow;

    }

    /**
     * A basic test of the factory before we begin.
     *
     * @return void
     */
    public function testModelsCanBeInstantiated()
    {
        $user = User::factory()->make();
        self::assertTrue($user instanceof User);
    }

    /**
     * Demonstrating the basic loop of following a user.
     * @return void
     */
    public function testFollowUserFeatures()
    {
        /** This user follows another user: */
        $this->user->followUser($this->user_to_follow);

        /** We can see that the one-way relationship exists. */
        self::assertTrue($this->user->isFollowingUser($this->user_to_follow));
        self::assertCount(1, $this->user_to_follow->followers());
        self::assertCount(1, $this->user->followingUsers());
        self::assertEquals($this->user_to_follow->username, $this->user->followingUsers()->first()->username);
        self::assertEquals($this->user->id, $this->user_to_follow->followers()->pluck('id')->first());

        /** And that the inverse relation is not automatically created. */
        self::assertFalse($this->user_to_follow->isFollowingUser($this->user));
        self::assertCount(0, $this->user->followers());

        /** Unfollowing the user: */
        $this->user->unfollowUser($this->user_to_follow);

        /** We can see that the relation is gone. */
        self::assertFalse($this->user->isFollowingUser($this->user_to_follow));
        self::assertCount(0, $this->user_to_follow->followers());
        self::assertCount(0, $this->user->followingUsers());

        /** Error handling: */

        /** A user can follow another user only once: */
        $this->user->followUser($this->user_to_follow);
        $this->user->followUser($this->user_to_follow);

        /** We will only have one follow interaction. */
        self::assertCount(1, $this->user_to_follow->followers());
        self::assertCount(1, $this->user->followingUsers());
    }
}
