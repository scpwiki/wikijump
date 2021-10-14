<?php

declare(strict_types=1);

namespace Tests\Feature;

use Illuminate\Auth\Access\AuthorizationException;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Support\Facades\Auth;
use Tests\TestCase;
use Wikijump\Models\User;
use Wikijump\Models\UserMessage;
use Wikijump\Policies\UserPolicy;

/**
 * Unit Tests for user-to-user messages.
 * @package Tests\Feature
 */
class UserMessageFeatureTest extends TestCase
{
    private User $sender;
    private User $recipient;
    private UserMessage $message;

    use RefreshDatabase;

    /**
     * Code inspectors miss the result of the create method binding to the desired type.
     * @noinspection PhpFieldAssignmentTypeMismatchInspection
     * Verified below:
     * @see testVerifyInstantiation()
     */
    protected function setUp(): void
    {
        parent::setUp();
        $this->sender = User::factory()->create();
        $this->recipient = User::factory()->create();
        $this->message = UserMessage::factory()->create([
            'from_user_id' => $this->sender->id,
            'to_user_id' => $this->recipient->id,
        ]);
    }

    /**
     * Preliminary assertions to verify our DB and factories are working.
     */
    public function testVerifyInstantiation()
    {
        self::assertTrue($this->sender instanceof User);
        self::assertTrue($this->recipient instanceof User);
        self::assertTrue($this->message instanceof UserMessage);
        self::assertTrue(is_string($this->message->subject));
        self::assertTrue(is_string($this->message->body));
        self::assertTrue(is_int($this->message->flags));
        self::assertTrue(is_int($this->message->from_user_id));
        self::assertTrue(is_int($this->message->to_user_id));
        self::assertTrue(is_string($this->message->id));
        self::assertFalse(is_numeric($this->message->id));
    }

    /**
     * This is primarily a test of the new User Policy as this is our
     *  first implementation of any authz policy.
     * @see UserPolicy
     */
    public function testNoMessagesWhenUserBlocksExist()
    {
        /** We need to authenticate as the sender for this test. */
        Auth::login($this->sender);

        /** Our initial assertions that the factory-sent message is there: */
        self::assertEquals(1, UserMessage::all()->count());
        self::assertEquals($this->sender->id, $this->message->sender->id);
        self::assertEquals($this->recipient->id, $this->message->recipient->id);

        /** Block the sender */
        $this->recipient->blockUser($this->sender);
        self::assertTrue($this->recipient->isBlockingUser($this->sender));

        /** We build a new message: */
        $newmessage = UserMessage::factory()->make([
            'from_user_id' => $this->sender->id,
            'to_user_id' => $this->recipient->id,
        ]);

        /** It should throw an exception. */
        self::expectException(AuthorizationException::class);
        self::expectErrorMessage('You are blocked by the recipient.');
        $newmessage->send();

        /** And no new messages have been created. */
        self::assertEquals(1, UserMessage::all()->count());
    }
}
