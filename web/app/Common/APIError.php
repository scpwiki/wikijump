<?php
declare(strict_types=1);

namespace Wikijump\Common;

use Exception;
use Illuminate\Http\Response;
use Wikijump\Common\Enum;

/** Enum for API error types. */
final class APIError extends Enum
{
    const BAD_SYNTAX = 'BAD_SYNTAX';
    const UNAUTHORIZED = 'UNAUTHORIZED';
    const FORBIDDEN = 'FORBIDDEN';
    const NOT_FOUND = 'NOT_FOUND';
    const CONFLICT = 'CONFLICT';

    const ACCOUNT_ALREADY_VERIFIED = 'ACCOUNT_ALREADY_VERIFIED';
    const ACCOUNT_NO_EMAIL = 'ACCOUNT_NO_EMAIL';
    const ALREADY_LOGGED_IN = 'ALREADY_LOGGED_IN';
    const FAILED_TO_UPDATE_PROFILE = 'FAILED_TO_UPDATE_PROFILE';
    const INVALID_ALLOW_MESSAGES = 'INVALID_ALLOW_MESSAGES';
    const INVALID_AVATAR = 'INVALID_AVATAR';
    const INVALID_EMAIL = 'INVALID_EMAIL';
    const INVALID_LANGUAGE_CODE = 'INVALID_LANGUAGE_CODE';
    const INVALID_PASSWORD = 'INVALID_PASSWORD';
    const INVALID_SESSION = 'INVALID_SESSION';
    const INVALID_SPECIFIER = 'INVALID_SPECIFIER';
    const INVALID_USERNAME = 'INVALID_USERNAME';
    const LOGIN_FAILED = 'LOGIN_FAILED';
    const NOT_LOGGED_IN = 'NOT_LOGGED_IN';
    const UNKNOWN_EMAIL = 'UNKNOWN_EMAIL';
    const UNKNOWN_USER = 'UNKNOWN_USER';
    const WRONG_PASSWORD = 'WRONG_PASSWORD';
    const SITE_NOT_FOUND = 'SITE_NOT_FOUND';
    const UNIMPLEMENTED = 'UNIMPLEMENTED';
    const INVALID_PAGE_TYPE = 'INVALID_PAGE_TYPE';
    const INVALID_PAGE_PATH = 'INVALID_PAGE_PATH';
    const PAGE_NOT_FOUND = 'PAGE_NOT_FOUND';

    /** Makes an a proper `Response` for when returning API errors.
     * @param int $status HTTP status code. Should be something in the 400 range.
     * @param string $error The error name, as a string. Needs to a value of one of the constants in this class.
     */
    public static function makeResponse(int $status, string $error)
    {
        if (!static::isValue($error)) {
            throw new Exception('Invalid error type');
        }
        return new Response(['error' => $error], $status);
    }
}
