<?php
declare(strict_types=1);

namespace Wikijump\Common;

use Exception;
use Illuminate\Http\Response;
use Wikijump\Common\Enum;

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

    public static function makeResponse(int $status, string $error)
    {
        if (!static::isValue($error)) {
            throw new Exception('Invalid error type');
        }
        return new Response(['error' => $error], $status);
    }
}
