<?php

namespace Ozone\Framework;

/**
 * Utility Class for providing unique strings.
 *
 */
class UniqueStrings
{
    public static function random_string(int $length): string
    {
        // Returns a string of double the requested length.
        $bytes = random_bytes($length);

        // Hexadecimal doubles the string length.
        // Slice for proper length with odd numbers.
        return substr(bin2hex($bytes), 0, $length);
    }
}
