<?php
declare(strict_types=1);

namespace Wikijump\Services\Deepwell;

use Exception;
use Wikijump\Common\Enum;

final class UserDetail extends Enum
{
    const IDENTITY = 'identity';
    const INFO = 'info';
    const PROFILE = 'profile';

    /**
     * Gives an ordering from UserDetails enums so they can be sorted.
     * Greater values means more details.
     *
     * @param string $value UserDetails enum value
     * @return int The order value
     * @throws Exception If the argument is not a valid enum value.
     */
    public static function getOrder(string $value): int
    {
        switch ($value) {
            case IDENTITY:
                return 0;
            case INFO:
                return 1;
            case PROFILE:
                return 2;
            default:
                throw new Exception("Invalid UserDetails enum value: $value");
        }
    }
}
