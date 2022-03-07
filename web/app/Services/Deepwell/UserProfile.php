<?php
declare(strict_types=1);

namespace Wikijump\Services\Deepwell;

use Carbon\Carbon;

/**
 * Helper class, containing only the fields added in the 'profile' details level.
 */
class UserProfile
{
    public ?string $real_name;
    public ?string $pronouns;
    public ?Carbon $birthday;
    public ?string $location;
    public array $links;

    public function __construct(object $raw_user)
    {
        $this->real_name = $raw_user->realName;
        $this->pronouns = $raw_user->pronouns;
        $this->birthday = $raw_user->birthday;
        $this->location = $raw_user->location;
        $this->links = $raw_user->links;
    }
}
