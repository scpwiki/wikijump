<?php
declare(strict_types=1);

namespace Wikijump\Services\Deepwell;

use Carbon\Carbon;

/**
 * Helper class, containing only the fields added in the 'info' details level.
 */
class UserInfo
{
    public ?string $about;
    public ?string $avatar;
    public ?string $signature;
    public ?Carbon $since;
    public ?Carbon $last_active;

    public function __construct(object $raw_user)
    {
        $this->about = $raw_user->about;
        $this->avatar = $raw_user->avatar;
        $this->signature = $raw_user->signature;
        $this->since = $raw_user->since;
        $this->last_active = $raw_user->lastActive;
    }
}
