<?php

namespace Wikijump\Models;

use Database\Seeders\UserSeeder;
use Illuminate\Contracts\Auth\MustVerifyEmail;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Relations\MorphOne;
use Illuminate\Foundation\Auth\User as Authenticatable;
use Illuminate\Notifications\Notifiable;
use Illuminate\Database\Eloquent\SoftDeletes;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Support\Facades\Config;
use Wikijump\Traits\HasSettings;

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
    protected array $casts = [
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

}
