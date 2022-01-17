<?php
declare(strict_types=1);

namespace Wikijump\Services\Deepwell\Models;

use Exception;
use Wikijump\Services\Deepwell\DeepwellService;

class User extends DeepwellModel
{
    // Fields and constructor
    private string $detail;
    private int $id;
    private string $username;
    private string $tiny_avatar;
    private int $karma;
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
    public function details(): string
    {
        return $this->details;
    }

    public function id(): int
    {
        return $this->id;
    }

    public function username(): string
    {
        return $this->username;
    }

    public function tiny_avatar(): string
    {
        return $this->tiny_avatar;
    }

    public function karma(): int
    {
        return $this->karma;
    }

    // Instance methods
    public function save(): void
    {
        throw new Exception('Not implemented yet!');
    }
}
