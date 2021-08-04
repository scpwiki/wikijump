<?php
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
use Wikijump\Helpers\InteractionType;
use Wikijump\Traits\HasInteractions;
use Wikijump\Traits\HasSettings;
use Wikijump\Traits\LegacyCompatibility;

/**
 * Class User
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
    public function avatarSm(): string
    {
        return '/common--images/avatars/'.floor($this->id/1000).'/'.$this->id.'/a16.png';
    }

    /**
     * Retrieve the path for the user's large avatar.
     * @return string
     */
    public function avatarLg(): string
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
     * Follow a user.
     * @param User $user_to_follow
     * @return Interaction
     */
    public function followUser(User $user_to_follow) : Interaction
    {
        return Interaction::add($this, InteractionType::USER_FOLLOWS_USER, $user_to_follow);
    }

    /**
     * Unfollow a user.
     * @param User $user_to_unfollow
     * @return int
     */
    public function unfollowUser(User $user_to_unfollow) : int
    {
        return Interaction::remove($this, InteractionType::USER_FOLLOWS_USER, $user_to_unfollow);
    }

    /**
     * Check if this user is following the target user.
     * @param User $user
     * @return bool
     */
    public function isFollowingUser(User $user) : bool
    {
        return Interaction::exists($this, InteractionType::USER_FOLLOWS_USER, $user);
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
     * Add a user to this user's contact list.
     * Note, all retrieve operations are bidirectional, so it doesn't matter
     *  who added whom.
     * @param User $user_to_add
     * @return Interaction|null
     */
    public function addContact(User $user_to_add): ?Interaction
    {
        return Interaction::add($this, InteractionType::USER_CONTACTS, $user_to_add);
    }

    /**
     * Remove a user from this user's contact list, and vice-versa.
     * @param User $user_to_remove
     * @return int|null
     */
    public function removeContact(User $user_to_remove): ?int
    {
        $mine = Interaction::remove($this, InteractionType::USER_CONTACTS, $user_to_remove);
        $theirs = Interaction::remove($user_to_remove, InteractionType::USER_CONTACTS, $this);
        return $mine + $theirs;
    }

    /**
     * Create a request to be added as a contact of another user.
     * @param User $user_to_request
     * @return Interaction|null
     */
    public function requestContact(User $user_to_request): ?Interaction
    {

        /** If this user already has a request for the target user, return null. */
        if(Interaction::exists($this, InteractionType::USER_CONTACT_REQUESTS, $user_to_request))
        {
            return null;
        }

        /** If the inverse request exists, create the contact relation. */
        if(Interaction::exists($user_to_request, InteractionType::USER_CONTACT_REQUESTS, $this))
        {
            return $this->approveContactRequest($user_to_request);
        }

        /** If a Contact relationship already exists, disallow another request. */
        if($this->isContact($user_to_request))
        {
            return null;
        }

        // TODO: More edge cases here around blocked users when that class comes.

        return Interaction::add($this, InteractionType::USER_CONTACT_REQUESTS, $user_to_request);
    }

    /**
     * Deny a request from another user to be added as a contact.
     * @param User $user_to_deny
     * @return int|null
     */
    public function denyContactRequest(User $user_to_deny): ?int
    {
        return Interaction::remove($user_to_deny, InteractionType::USER_CONTACT_REQUESTS, $this);
    }

    /**
     * Cancel a pending request to be added as another user's contact.
     * @param User $user_to_cancel
     * @return int|null
     */
    public function cancelContactRequest(User $user_to_cancel): ?int
    {
        return Interaction::remove($this, InteractionType::USER_CONTACT_REQUESTS, $user_to_cancel);
    }

    /**
     * Approve a request to be added as a contact of another user.
     * @param User $user_to_approve
     * @return Interaction|null
     */
    public function approveContactRequest(User $user_to_approve): ?Interaction
    {
        Interaction::remove($user_to_approve, InteractionType::USER_CONTACT_REQUESTS, $this);
        return $this->addContact($user_to_approve);
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
            Interaction::exists($this, InteractionType::USER_CONTACTS, $user) ||
            Interaction::exists($user, InteractionType::USER_CONTACTS, $this)
        );
    }

}
