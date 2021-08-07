<?php

/** @noinspection PhpUnused */
declare(strict_types=1);

namespace Wikijump\Models;

use Database\Seeders\UserSeeder;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Foundation\Auth\User as Authenticatable;
use Illuminate\Notifications\Notifiable;
use Illuminate\Database\Eloquent\SoftDeletes;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Support\Facades\Config;
use Illuminate\Support\Facades\DB;
use Wikijump\Helpers\InteractionType;
use Wikijump\Traits\HasInteractions;
use Wikijump\Traits\HasSettings;
use Wikijump\Traits\LegacyCompatibility;

/**
 * Class User
 * @property int id
 * @package Wikijump\Models
 * @mixin Builder
 */
class User extends Authenticatable
{
    use HasFactory;
    use Notifiable;
    use SoftDeletes;
    use HasSettings;
    use LegacyCompatibility;
    use HasInteractions;

    /**
     * These are service accounts added by the UserSeeder. They're used during
     * the generation of various pages, threads, and so on. This is their ID.
     * @see UserSeeder
     */
    public const AUTOMATIC_USER = 2;
    public const ANONYMOUS_USER = 3;

    /**
     * @var string
     */
    private string $language;

    /**
     * The attributes that are mass assignable.
     *
     * @var array
     */
    protected $fillable = [
        'username',
        'email',
        'password',
    ];

    /**
     * The model's default values for attributes.
     *
     * @var array
     */
    protected $attributes = [

    ];

    /**
     * The attributes that should be hidden for arrays.
     *
     * @var array
     */
    protected $hidden = [
        'password',
        'remember_token',
        'two_factor_secret',
        'two_factor_recovery_codes'
    ];

    /**
     * The attributes that should be cast to native types.
     *
     * @var array
     */
    protected $casts = [
        'email_verified_at' => 'datetime',
    ];

    /**
     * Retrieve the path for the user's small avatar.
     * @return string
     */
    public function avatarSmall(): string
    {
        return '/common--images/avatars/'.floor($this->id/1000).'/'.$this->id.'/a16.png';
    }

    /**
     * Retrieve the path for the user's large avatar.
     * @return string
     */
    public function avatarLarge(): string
    {
        return '/common--images/avatars/'.floor($this->id/1000).'/'.$this->id.'/a48.png';
    }

    /**
     * Retrieve the list of default values for user settings.
     * @return array
     */
    public static function defaults() : array
    {
        return Config::get('wikijump.defaults.user');
    }

    /**
     * Retrieve the users this user is following.
     * @return Collection<User>
     */
    public function followingUsers() : Collection
    {
        return $this->my(InteractionType::USER_FOLLOWS_USER);
    }

    /**************************************************
     * Following Users
     *************************************************/


    /**
     * Follow a user.
     * @param User $user_to_follow
     * @return bool
     */
    public function followUser(User $user_to_follow) : bool
    {
        if ($this->isFollowingUser($user_to_follow) === false)
        {
            return Interaction::create($this, InteractionType::USER_FOLLOWS_USER, $user_to_follow);
        }
        return false;
    }

    /**
     * Retrieve the users following this user.
     *
     * @return Collection<User>
     */
    public function followers() : Collection
    {
        return $this->their(InteractionType::USER_FOLLOWS_USER);
    }

    /**
     * Check if this user is following the target user.
     * @param User $user
     * @return bool
     */
    public function isFollowingUser(User $user) : bool
    {
        return Interaction::check($this, InteractionType::USER_FOLLOWS_USER, $user);
    }

    /**
     * Unfollow a user.
     * @param User $user_to_unfollow
     * @return bool
     */
    public function unfollowUser(User $user_to_unfollow) : bool
    {
        return Interaction::remove($this, InteractionType::USER_FOLLOWS_USER, $user_to_unfollow);
    }

    /**************************************************
     * Contacts System
     *************************************************/

    /**
     * Add a user to this user's contact list.
     * Note, all retrieve operations are bidirectional, so it doesn't matter
     *  who added whom.
     * @param User $user_to_add
     * @return bool
     */
    public function addContact(User $user_to_add): bool
    {
        return Interaction::create($this, InteractionType::USER_CONTACTS, $user_to_add);
    }

    /**
     * Create a request to be added as a contact of another user.
     * @param User $user_to_request
     * @return bool
     */
    public function requestContact(User $user_to_request): bool
    {

        /** If a block is in place on either side, disallow a request. */
        if($this->userBlockExists($user_to_request))
        {
            return false;
        }

        /** If this user already has a request for the target user, return null. */
        if(Interaction::check($this, InteractionType::USER_CONTACT_REQUESTS, $user_to_request))
        {
            return false;
        }

        /** If the inverse request exists, create the contact relation. */
        if(Interaction::check($user_to_request, InteractionType::USER_CONTACT_REQUESTS, $this))
        {
            return $this->approveContactRequest($user_to_request);
        }

        /** If a Contact relationship already exists, disallow another request. */
        if($this->isContact($user_to_request))
        {
            return false;
        }

        return Interaction::create($this, InteractionType::USER_CONTACT_REQUESTS, $user_to_request);
    }

    /**
     * Approve a request to be added as a contact of another user.
     * @param User $user_to_approve
     * @return bool
     */
    public function approveContactRequest(User $user_to_approve): bool
    {
        DB::transaction(function () use ($user_to_approve) {
            Interaction::remove($user_to_approve, InteractionType::USER_CONTACT_REQUESTS, $this);
            return $this->addContact($user_to_approve);
        });

        return false;
    }

    /**
     * Retrieve all contacts for a user.
     * @return Collection<User>
     */
    public function contacts() : Collection
    {
        return $this->either(InteractionType::USER_CONTACTS);
    }

    /**
     * Retrieve a collection of users that have requested to be added as a contact.
     * @return Collection<User>
     */
    public function viewIncomingContactRequests(): Collection
    {
        return $this->their(InteractionType::USER_CONTACT_REQUESTS);
    }

    /**
     * View all pending contact requests this user has made.
     * @return Collection<User>
     */
    public function viewOutgoingContactRequests(): Collection
    {
        return $this->my(InteractionType::USER_CONTACT_REQUESTS);
    }

    /**
     * Check if a user is a contact of this user. Bidirectional.
     * @param User $user
     * @return bool
     */
    public function isContact(User $user): bool
    {
        return (
            Interaction::check($this, InteractionType::USER_CONTACTS, $user)
            || Interaction::check($user, InteractionType::USER_CONTACTS, $this)
        );
    }

    /**
     * Remove a user from this user's contact list, and vice-versa.
     * @param User $user_to_remove
     * @return bool
     */
    public function removeContact(User $user_to_remove): bool
    {
        return(
            Interaction::remove($this, InteractionType::USER_CONTACTS, $user_to_remove)
            || Interaction::remove($user_to_remove, InteractionType::USER_CONTACTS, $this)
        );
    }

    /**
     * Deny a request from another user to be added as a contact.
     * @param User $user_to_deny
     * @return bool
     */
    public function denyContactRequest(User $user_to_deny): bool
    {
        return Interaction::remove($user_to_deny, InteractionType::USER_CONTACT_REQUESTS, $this);
    }

    /**
     * Deny a request from another user to be added as a contact, and block them from further interaction.
     * @param User $user_to_deny
     * @return bool
     */
    public function denyContactRequestAndBlock(User $user_to_deny): bool
    {
        DB::transaction(function() use ($user_to_deny) {
            $this->blockUser($user_to_deny);
            return Interaction::remove($user_to_deny, InteractionType::USER_CONTACT_REQUESTS, $this);
        });

        return false;
    }

    /**
     * Cancel a pending request to be added as another user's contact.
     * @param User $user_to_cancel
     * @return bool
     */
    public function cancelContactRequest(User $user_to_cancel): bool
    {
        return Interaction::remove($this, InteractionType::USER_CONTACT_REQUESTS, $user_to_cancel);
    }

    /**************************************************
     * User Blocks User
     *************************************************/

    /**
     * This user blocks the target user.
     * @param User $user_to_block
     * @param string $reason
     * @return bool
     */
    public function blockUser(User $user_to_block, string $reason = '') : bool
    {
        /** Do not add a second block for the same target user. */
        if($this->isBlockingUser($user_to_block)) { return false; }

        /** Once the block occurs, remove contact and any pending requests. */
        DB::transaction(function () use($user_to_block, $reason) {
            $this->cancelContactRequest($user_to_block);
            $this->denyContactRequest($user_to_block);
            $this->removeContact($user_to_block);

            $reason = filter_var($reason, FILTER_SANITIZE_STRING);
            return Interaction::create($this, InteractionType::USER_BLOCKS_USER, $user_to_block, ['reason' => $reason]);
        });

        return false;
    }

    public function getBlock(User $user) : ?Interaction
    {
        return Interaction::retrieve($this, InteractionType::USER_BLOCKS_USER, $user);
    }

    /**
     * List all users blocked by this user.
     * @return Collection
     */
    public function viewBlockedUsers() : Collection
    {
        return $this->my(InteractionType::USER_BLOCKS_USER);
    }

    /**
     * Check if this user is blocking the target user.
     * @param User $user
     * @return bool
     */
    public function isBlockingUser(User $user) : bool
    {
        return Interaction::check($this, InteractionType::USER_BLOCKS_USER, $user);
    }

    /**
     * Check if this user is blocked by the target user.
     * @param User $user
     * @return bool
     */
    public function isBlockedByUser(User $user) : bool
    {
        return Interaction::check($user, InteractionType::USER_BLOCKS_USER, $this);
    }

    /**
     * Check if either this user or the target user is blocking the other.
     * @param User $user
     * @return bool
     */
    public function userBlockExists(User $user) : bool
    {
        return (
            Interaction::check($this, InteractionType::USER_BLOCKS_USER, $user)
            || Interaction::check($user, InteractionType::USER_BLOCKS_USER, $this)
        );
    }

    /**
     * Update the reason for a block on a user.
     * @param User $user_to_block
     * @param string $reason
     * @return bool
     */
    public function updateUserBlock(User $user_to_block, string $reason = '') : bool
    {
        if($this->isBlockingUser($user_to_block) === false) { return false; }

        $reason = filter_var($reason, FILTER_SANITIZE_STRING);
        return Interaction::set($this, InteractionType::USER_BLOCKS_USER, $user_to_block, ['reason' => $reason]);
    }

    /**
     * Remove this user's block on a target user.
     * @param User $user_to_unblock
     * @return bool
     */
    public function unblockUser(User $user_to_unblock) : bool
    {
        if($this->isBlockingUser($user_to_unblock) === false) { return false; }

        return Interaction::remove($this, InteractionType::USER_BLOCKS_USER, $user_to_unblock);
    }
}
