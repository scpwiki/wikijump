<?php
declare(strict_types=1);

/** This file is a set of default helper functions to wrap code in. */

use intl\MessageFormatter;

/**
 * Check if the error code from Postgres matches a string (actually a const).
 * @param Throwable $e
 * @param string $code
 * @return bool
 */
function pg_is_error(Throwable $e, string $code): bool
{
    return (string) $e->getCode() === $code;
}

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
        Log::warn("Unable to find message '$key' in locale '$locale'");
    }

    return MessageFormatter::formatMessage($locale, $message, $values);
}
