<?php
declare(strict_types=1);

/**
 * This file is for helper functions (like helpers.php), but
 * for functions which have the same name as Laravel (or other vendors)
 * helpers, so must be loaded in a special way to prevent conflicts.
 */

use intl\MessageFormatter;

/**
 * Helper for localizing strings, overwriting Laravel's __() function.
 * It has the same usage pattern as __(), but uses our localization system
 * rather than Laravel's.
 *
 * @param string $key The translation key to look up.
 * @param array $values Optional values to substitute as part of message formatting.
 */
function __(string $key, array $values = [])
{
    $locale = App::currentLocale();
    $message = gettext($key);
    if ($message === $key) {
        Log::warning("Unable to find message '$key' in locale '$locale'");
    }

    return MessageFormatter::formatMessage($locale, $message, $values);
}
