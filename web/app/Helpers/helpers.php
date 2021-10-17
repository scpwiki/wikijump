<?php
declare(strict_types=1);

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
