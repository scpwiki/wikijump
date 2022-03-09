<?php
declare(strict_types=1);

namespace Wikijump\Services\Deepwell\Models;

use Wikijump\Services\Deepwell\DeepwellService;
use Wikijump\Services\Deepwell\UserDetail;
use Wikijump\Services\Deepwell\UserInfo;
use Wikijump\Services\Deepwell\UserProfile;

class User extends DeepwellModel
{
    // Fields and constructor
    public string $detail;
    public int $id;
    public string $username;
    public ?string $tiny_avatar;
    public int $karma;
    public ?UserInfo $info;
    public ?UserProfile $profile;

    public function __construct(object $raw_user, string $detail)
    {
        $this->detail = $detail;
        $this->id = $raw_user->id;
        $this->username = $raw_user->username;
        $this->tiny_avatar = $raw_user->tinyavatar;
        $this->karma = $raw_user->karma;

        $detail_order = UserDetail::getOrder($detail);
        if ($detail_order >= UserDetail::getOrder(UserDetail::INFO)) {
            $this->info = new UserInfo($raw_user);
        } else {
            $this->info = null;
        }

        if ($detail_order >= UserDetail::getOrder(UserDetail::PROFILE)) {
            $this->profile = new UserProfile($raw_user);
        } else {
            $this->profile = null;
        }
    }

    // Fetch methods
    public static function findId(int $user_id): ?User
    {
        return DeepwellService::getInstance()->getUserById($user_id);
    }

    public static function findSlug(string $user_slug): ?User
    {
        return DeepwellService::getInstance()->getUserBySlug($user_slug);
    }

    // Getters
    public function id(): int
    {
        return $this->id;
    }

    // API utility methods
    public function toApiArray(bool $avatars = true): array
    {
        $output = [
            'id' => $this->id,
            'username' => $this->username,
            'tinyavatar' => $avatars ? $this->tiny_avatar : null,
            'karma' => $this->karma,
            'role' => 'registered', // TODO
        ];

        if ($this->info) {
            $output = array_merge($output, [
                'about' => $this->info->about,
                'avatar' => $this->info->avatar,
                'signature' => $this->info->signature,
                'since' => $this->info->since,
                'lastActive' => $this->info->last_active,
            ]);
        }

        if ($this->profile) {
            $output = array_merge($output, [
                'realname' => $this->profile->real_name,
                'pronouns' => $this->profile->pronouns,
                'birthday' => $this->profile->birthday,
                'location' => $this->profile->location,
                'links' => $this->profile->links,
            ]);
        }

        return $output;
    }
}
