<?php

declare(strict_types=1);

namespace Tests\Unit;

use Illuminate\Foundation\Testing\RefreshDatabase;
use Tests\TestCase;
use Wikijump\Models\User;
use Wikijump\Models\UserMessage;

/**
 * Unit Tests for user-to-user messages.
 * @package Tests\Unit
 */
class UserMessageTest extends TestCase
{

    use RefreshDatabase;

    private User $sender;
    private User $recipient;
    private UserMessage $message;
    private int $systemUsers;

    /**
     * Code inspectors miss the result of the save method binding to the desired type.
     * @noinspection PhpFieldAssignmentTypeMismatchInspection
     * Verified below:
     * @see testVerifyInstantiation()
     */
    protected function setUp(): void
    {
        parent::setUp();

        /** The users auto-generated as part of seeding, to verify counts from any new testing. */
        $this->systemUsers = User::all()->count();

        $this->sender = User::factory()->create();

        $this->recipient = User::factory()->create();

        $this->message = UserMessage::factory()->create([
            'from_user_id' => $this->sender->id,
            'to_user_id' => $this->recipient->id
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

    public function testVerifyDatabasePopulating()
    {
        self::assertEquals($this->systemUsers + 2, User::all()->count());
        self::assertEquals(1, UserMessage::all()->count());
        self::assertEquals($this->sender->id, $this->message->sender->id);
        self::assertEquals($this->recipient->id, $this->message->recipient->id);
    }

    /**
     * Unit tests follow:
     */

    public function testUnmarkAsDraft()
    {
        $this->message->setFlag(UserMessage::MESSAGE_DRAFT);
        self::assertTrue($this->message->getFlag(UserMessage::MESSAGE_DRAFT));
        $this->message->unmarkAsDraft();
        self::assertFalse($this->message->getFlag(UserMessage::MESSAGE_DRAFT));
    }

    public function testUnarchiveMessage()
    {
        $this->message->setFlag(UserMessage::MESSAGE_ARCHIVED);
        self::assertTrue($this->message->getFlag(UserMessage::MESSAGE_ARCHIVED));
        $this->message->unarchiveMessage();
        self::assertFalse($this->message->getFlag(UserMessage::MESSAGE_ARCHIVED));
    }

    public function testScopeArchive()
    {
        self::assertCount(0, UserMessage::archive($this->recipient)->get());
        $this->message->archiveMessage();
        self::assertCount(1, UserMessage::archive($this->recipient)->get());
    }

    public function testSender()
    {
        self::assertTrue($this->message->sender instanceof User);
        self::assertEquals($this->sender->id, $this->message->sender->id);
    }

    public function testMarkAsRead()
    {
        self::assertFalse($this->message->getFlag(UserMessage::MESSAGE_READ));
        $this->message->markAsRead();
        self::assertTrue($this->message->getFlag(UserMessage::MESSAGE_READ));
    }

    public function testMarkAsDraft()
    {
        self::assertFalse($this->message->getFlag(UserMessage::MESSAGE_DRAFT));
        $this->message->markAsDraft();
        self::assertTrue($this->message->getFlag(UserMessage::MESSAGE_DRAFT));
    }

    public function testRecipient()
    {
        self::assertTrue($this->message->recipient instanceof User);
        self::assertEquals($this->recipient->id, $this->message->recipient->id);
    }

    public function testArchiveMessage()
    {
        self::assertFalse($this->message->getFlag(UserMessage::MESSAGE_ARCHIVED));
        $this->message->archiveMessage();
        self::assertTrue($this->message->getFlag(UserMessage::MESSAGE_ARCHIVED));
    }

    public function testUnstarMessage()
    {
        $this->message->starMessage();
        self::assertTrue($this->message->getFlag(UserMessage::MESSAGE_STARRED));
        $this->message->unstarMessage();
        self::assertFalse($this->message->getFlag(UserMessage::MESSAGE_STARRED));
    }

    public function testScopeUnread()
    {
        self::assertCount(1, UserMessage::unread($this->recipient)->get());
        $this->message->markAsRead();
        self::assertCount(0, UserMessage::unread($this->recipient)->get());
    }

    public function testScopeInbox()
    {
        self::assertCount(1, UserMessage::inbox($this->recipient)->get());
        $this->message->delete();
        self::assertCount(0, UserMessage::inbox($this->recipient)->get());
    }

    public function testScopeDrafts()
    {
        self::assertCount(0, UserMessage::drafts($this->sender)->get());
        $this->message->markAsDraft();
        self::assertCount(1, UserMessage::drafts($this->sender)->get());
    }

    public function testMarkAsUnread()
    {
        $this->message->markAsRead();
        self::assertCount(0, UserMessage::unread($this->recipient)->get());
        $this->message->markAsUnread();
        self::assertCount(1, UserMessage::unread($this->recipient)->get());
    }

    public function testStarMessage()
    {
        self::assertFalse($this->message->getFlag(UserMessage::MESSAGE_STARRED));
        $this->message->starMessage();
        self::assertTrue($this->message->getFlag(UserMessage::MESSAGE_STARRED));
    }

    public function testScopeStarred()
    {
        self::assertCount(0, UserMessage::starred($this->recipient)->get());
        $this->message->starMessage();
        self::assertCount(1, UserMessage::starred($this->recipient)->get());
    }
}
