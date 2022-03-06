<?php
declare(strict_types=1);

use Illuminate\Http\Response;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Facades\URL;
use Wikijump\Common\APIError;

/**
 * This file is a set of default helper functions to wrap code in.
 */

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

if (!function_exists('_')) {
    /**
     * Legacy helper for Ozone's gettext() functions.
     *
     * @deprecated Use __() instead.
     */
    function _(string $key): string
    {
        Log::warning('Use of deprecated _() function');
        return __($key);
    }
}

/**
 * Returns the previous URL for the session.
 * Will return an empty string if the previous URL is the same as
 * the current one.
 */
function previousUrl(): string
{
    $url = URL::previous();

    // cut off query, which we can't keep unfortunately
    // the query string is written by nginx and is usually some giant mess
    if (strpos($url, '?') !== false) {
        $url = substr($url, 0, strpos($url, '?'));
    }

    if (URL::current() === $url) {
        return '';
    }

    return $url;
}

/**
 * Shorthand for making an API error response.
 * @param int $status HTTP status code.
 * @param string $error The error to throw. Must be a value of `APIError`.
 */
function apierror(int $status, string $error): Response
{
    return APIError::makeResponse($status, $error);
}
