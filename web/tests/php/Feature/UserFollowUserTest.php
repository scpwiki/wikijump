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

    /**
     * A basic test of the factory before we begin.
     *
     * @return void
     */
    public function testModelsCanBeInstantiated()
    {
        $user = User::factory()->make();
        UserFollowUserTest::assertTrue($user instanceof User);
    }

    /**
     * Demonstrating the basic loop of following a user.
     * @return void
     */
    public function testAUserCanFollowAnotherUser()
    {
        /**
         * @var User $user
         * @var User $user_to_follow
         */
        $user = User::factory()->make();
        $user->save();
        $user_to_follow = User::factory()->make();
        $user_to_follow->save();

        $user->followUser($user_to_follow);

        UserFollowUserTest::assertTrue($user->isFollowingUser($user_to_follow));
        UserFollowUserTest::assertCount(1, $user_to_follow->followers());
        UserFollowUserTest::assertEquals($user->id, $user_to_follow->followers()->pluck('id')->first());

        $user->unfollowUser($user_to_follow);

        UserFollowUserTest::assertFalse($user->isFollowingUser($user_to_follow));
        UserFollowUserTest::assertCount(0, $user_to_follow->followers());
    }
}
