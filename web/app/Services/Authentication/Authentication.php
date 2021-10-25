<?php

declare(strict_types=1);

namespace Wikijump\Services\Authentication;

use Wikijump\Models\User;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Hash;
use Illuminate\Validation\ValidationException;

/** Static class holding methods used for authenticating users. */
final class Authentication
{
    private function __construct()
    {
    }

    /**
     * Validates the credentials inside of a request. Doesn't attempt any sort
     * of authentication, just makes sure that the request is correctly structured.
     *
     * @param Request $request The request containing user credentials.
     */
    public static function validate(Request $request): ?array
    {
        $credentials = null;

        try {
            $credentials = $request->validate([
                'login' => 'required|string',
                'password' => 'required|string',
                'remember' => 'sometimes|boolean',
            ]);
        } catch (ValidationException $err) {
            return null;
        }

        return $credentials;
    }

    /**
     * Returns a user based on the type of login specifier, or null if the
     * user can't be found.
     *
     * @param string $specifier
     *    The login specifier, which is either a username or an email.
     */
    public static function userFromSpecifier(string $specifier): ?User
    {
        $is_email = filter_var($specifier, FILTER_VALIDATE_EMAIL);

        $user = $is_email
            ? User::where('email', $specifier)->first()
            : User::where('username', $specifier)->first();

        return $user;
    }

    /**
     * Takes an authentication request and returns an `AuthenticationResult`.
     * Use the `ok()` method of the result to see if a user was found.
     * If a user was found, call the `user()` method to get the user object.
     * If a user wasn't found, call the `error()` method to get the error.
     * The error will be an enum of `AuthenticationError`.
     *
     * @param Request $request The request containing user credentials.
     */
    public static function authenticate(Request $request): AuthenticationResult
    {
        $credentials = self::validate($request);

        // credentials were not structured correctly
        if ($credentials === null) {
            return new AuthenticationResult(AuthenticationError::FAILED_TO_VALIDATE);
        }

        $login = $credentials['login'];
        $password = $credentials['password'];
        $user = self::userFromSpecifier($login);

        // user wasn't found
        if ($user === null) {
            return new AuthenticationResult(AuthenticationError::INVALID_SPECIFIER);
        }

        // check their password
        if (!Hash::check($password, $user->password)) {
            return new AuthenticationResult(AuthenticationError::INVALID_PASSWORD);
        }

        return new AuthenticationResult($user);
    }
}
