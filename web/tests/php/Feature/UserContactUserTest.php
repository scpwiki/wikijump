<?php
declare(strict_types=1);

namespace Tests\Feature;

use Illuminate\Foundation\Testing\RefreshDatabase;
use Tests\TestCase;
use Wikijump\Models\User;

/**
 * Class UserContactUserTest
 * A feature test for User methods involving Contacts.
 * @package Tests\Feature
 */
class UserContactUserTest extends TestCase
{

    use RefreshDatabase;

    private User $user;
    private User $user_to_contact;

    protected function setUp(): void
    {
        parent::setUp();
        $user = User::factory()->make();
        $user->save();
        $user_to_contact = User::factory()->make();
        $user_to_contact->save();

        $this->user = $user;
        $this->user_to_contact = $user_to_contact;

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
     * Demonstrating the basic loop of adding a user to contacts.
     * @return void
     */
    public function testContactUserFeatures()
    {
        /** Contact Request: */
        $this->user->requestContact($this->user_to_contact);

        /** The target user can see the incoming request. */
        self::assertCount(1, $this->user_to_contact->viewIncomingContactRequests());
        self::assertEquals(
            $this->user->username,
            $this->user_to_contact->viewIncomingContactRequests()->first()->username
        );

        /** The requesting user can see the pending request. */
        self::assertCount(1, $this->user->viewOutgoingContactRequests());
        self::assertEquals(
            $this->user_to_contact->username,
            $this->user->viewOutgoingContactRequests()->first()->username
        );

        /** There can be only one contact request pending. */
        self::assertNull($this->user->requestContact($this->user_to_contact));
        self::assertCount(1, $this->user_to_contact->viewIncomingContactRequests());
        self::assertCount(1, $this->user->viewOutgoingContactRequests());

        /** Denial of a request: */
        $this->user_to_contact->denyContactRequest($this->user);

        /** The target user no longer has an incoming request. */
        self::assertCount(0, $this->user_to_contact->viewIncomingContactRequests());

        /** The requesting user no longer has a pending request. */
        self::assertCount(0, $this->user->viewOutgoingContactRequests());

        /** No change is made to the contacts list. */
        self::assertCount(0, $this->user->contacts());
        self::assertCount(0, $this->user_to_contact->contacts());

        /** Cancelling a request: */
        $this->user->requestContact($this->user_to_contact);
        $this->user->cancelContactRequest($this->user_to_contact);

        /** The target user no longer has an incoming request. */
        self::assertCount(0, $this->user_to_contact->viewIncomingContactRequests());

        /** The requesting user no longer has a pending request. */
        self::assertCount(0, $this->user->viewOutgoingContactRequests());

        /** No change is made to the contacts list. */
        self::assertCount(0, $this->user->contacts());
        self::assertCount(0, $this->user_to_contact->contacts());

        /** Approval of a request: */
        $this->user->requestContact($this->user_to_contact);
        $this->user_to_contact->approveContactRequest($this->user);

        /** The bidirectional contact is established. */
        self::assertCount(1, $this->user->contacts());
        self::assertCount(1, $this->user_to_contact->contacts());
        self::assertEquals($this->user_to_contact->username,
            $this->user->contacts()->first()->username
        );
        self::assertEquals($this->user->username,
                           $this->user_to_contact->contacts()->first()->username
        );
        self::assertTrue($this->user->isContact($this->user_to_contact));
        self::assertTrue($this->user_to_contact->isContact($this->user));

        /** Removal of a contact: */
        $this->user->removeContact($this->user_to_contact);

        /** The bidirectional contact is removed. */
        self::assertCount(0, $this->user->contacts());
        self::assertCount(0, $this->user_to_contact->contacts());
        self::assertFalse($this->user->isContact($this->user_to_contact));
        self::assertFalse($this->user_to_contact->isContact($this->user));

        /** Edge cases around pending requests: */
        $this->user->requestContact($this->user_to_contact);

        /** If there are simultaneous request in each direction, the request is approved. */
        $this->user_to_contact->requestContact($this->user);
        self::assertCount(1, $this->user->contacts());
        self::assertCount(1, $this->user_to_contact->contacts());
        self::assertEquals($this->user_to_contact->username,
                           $this->user->contacts()->first()->username
        );
        self::assertEquals($this->user->username,
                           $this->user_to_contact->contacts()->first()->username
        );
        self::assertTrue($this->user->isContact($this->user_to_contact));
        self::assertTrue($this->user_to_contact->isContact($this->user));

        /** If two users are already contacts, they can not send new requests. */
        $this->user->requestContact($this->user_to_contact);
        self::assertCount(0, $this->user_to_contact->viewIncomingContactRequests());
        self::assertCount(0, $this->user->viewOutgoingContactRequests());
        $this->user_to_contact->requestContact($this->user);
        self::assertCount(0, $this->user->viewIncomingContactRequests());
        self::assertCount(0, $this->user_to_contact->viewOutgoingContactRequests());

    }
}
