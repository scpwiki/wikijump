<?php
declare(strict_types=1);

namespace Tests\Feature;

use Illuminate\Foundation\Testing\RefreshDatabase;
use Tests\TestCase;
use Wikijump\Models\User;

/**
 * Class UserBlockUserTest
 * A feature test for User methods involving User blocks.
 * @package Tests\Feature
 */
class UserBlockUserTest extends TestCase
{

    use RefreshDatabase;

    private User $user;
    private User $user_to_block;

    protected function setUp(): void
    {
        parent::setUp();
        $user = User::factory()->make();
        $user->save();
        $user_to_block = User::factory()->make();
        $user_to_block->save();

        $this->user = $user;
        $this->user_to_block = $user_to_block;
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
     * Demonstrating user-blocking functionality and its practical effects.
     * @return void
     */
    public function testBlockUserFeatures()
    {
        /** Block Applied: */
        $this->user->blockUser($this->user_to_block);

        /** Simple checks for existence of the block. */
        self::assertCount(1, $this->user->viewBlockedUsers());
        self::assertTrue($this->user->isBlockingUser($this->user_to_block));
        self::assertTrue($this->user_to_block->isBlockedByUser($this->user));
        self::assertTrue($this->user->userBlockExists($this->user_to_block));

        /** While a block is in place, some interactions are forbidden. */

        /** The blocked user attempts to send a contact request: */
        $this->user_to_block->requestContact($this->user);

        /** No request is generated. */
        self::assertCount(0, $this->user->viewIncomingContactRequests());

        /** The inverse is true, the blocking user cannot contact request a user they're blocking. */
        $this->user->requestContact($this->user_to_block);
        self::assertCount(0, $this->user_to_block->viewIncomingContactRequests());

        /** Removing a block: */
        $this->user->unblockUser($this->user_to_block);

        /** Existence checks for the block now fail. */
        self::assertCount(0, $this->user->viewBlockedUsers());
        self::assertFalse($this->user->isBlockingUser($this->user_to_block));
        self::assertFalse($this->user_to_block->isBlockedByUser($this->user));
        self::assertFalse($this->user->userBlockExists($this->user_to_block));

        /** Additional behaviors: */

        /** A user can deny a contact request and block the requesting user at the same time. */
        $this->user_to_block->requestContact($this->user);
        self::assertCount(1, $this->user->viewIncomingContactRequests());
        $this->user->denyContactRequestAndBlock($this->user_to_block);

        /** The request is gone and the blocked user cannot make a new one. */
        self::assertCount(0, $this->user->viewIncomingContactRequests());
        self::assertTrue($this->user->isBlockingUser($this->user_to_block));
        $this->user_to_block->requestContact($this->user);
        self::assertCount(0, $this->user->viewIncomingContactRequests());

        /** Reset */
        $this->user->unblockUser($this->user_to_block);

        /** Blocking a user that was previously a contact removes the contact relationship as well: */
        $this->user->addContact($this->user_to_block);
        self::assertTrue($this->user->isContact($this->user_to_block));
        $this->user->blockUser($this->user_to_block);
        self::assertFalse($this->user->isContact($this->user_to_block));

        /** Reset */
        $this->user->unblockUser($this->user_to_block);

        /** A reason for the block can be set, retrieved, and updated. */
        $this->user->blockUser($this->user_to_block, 'foo');
        $block = $this->user->getBlock($this->user_to_block);
        self::assertEquals('foo', $block->metadata['reason']);

        $this->user->updateUserBlock($this->user_to_block, 'bar');
        $block = $this->user->getBlock($this->user_to_block);
        self::assertEquals('bar', $block->metadata['reason']);
    }
}
