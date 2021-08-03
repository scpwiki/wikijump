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
     * Retrieve the list of default values for user settings.
     * @return array
     */
    public static function defaults() : array
    {
        return Config::get('wikijump.defaults.user');
    }

    /**
     * Retrieve the users this user is following.
     * @return Collection
     */
    public function followingUsers() : Collection
    {
        $list = $this->morphMany(Interaction::class, 'setter')
            ->where('interaction_type', InteractionType::USER_FOLLOWS_USER)
            ->pluck('target_id')->toArray();
        return User::whereIn('id', $list)->get();
    }

    /**
     * Retrieve the users following this user.
     * @return Collection
     */
    public function followers() : Collection
    {
        $list = $this->morphMany(Interaction::class, 'target')
            ->where('interaction_type', InteractionType::USER_FOLLOWS_USER)
            ->pluck('setter_id')->toArray();
        return User::whereIn('id', $list)->get();
    }

    /**
     * Follow a user.
     * @param User $userToFollow
     * @return Interaction
     */
    public function followUser(User $userToFollow) : Interaction
    {
        return Interaction::add($this, InteractionType::USER_FOLLOWS_USER, $userToFollow);
    }

    /**
     * Unfollow a user.
     * @param User $userToUnfollow
     * @return int
     */
    public function unfollowUser(User $userToUnfollow) : int
    {
        return Interaction::remove($this, InteractionType::USER_FOLLOWS_USER, $userToUnfollow);
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

}
