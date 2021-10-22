<?php

declare(strict_types=1);

namespace Wikijump\Actions\Fortify;

use App\Models\User;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Hash;
use Illuminate\Validation\ValidationException;

/** Static class holding methods used for authenticating users. */
class AuthenticateUser
{
    private function __construct()
    {
    }

    /**
     * Validates the credentials inside of a request. Doesn't attempt any sort
     * of authentication, just makes sure that the request is correctly structured.
     * @param Request $request The request containing user credentials.
     */
    static function validate(Request $request): ?array
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
     * @param string $specifier
     *    The login specifier, which is either a username or an email.
     */
    static function userFromSpecifier(string $specifier): ?User
    {
        $is_email = filter_var($specifier, FILTER_VALIDATE_EMAIL);

        $user = null;

        if ($is_email) {
            $user = User::where('email', $specifier)->first();
        } else {
            $user = User::where('username', $specifier)->first();
        }

        return $user;
    }

    /**
     * Takes an authentication request and returns a user if the request contains
     * valid authentication credentials, but null otherwise.
     * @param Request $request The request containing user credentials.
     */
    static function handle(Request $request): ?User
    {
        $credentials = self::validate($request);

        if ($credentials === null) {
            return null;
        }

        $login = $credentials['login'];
        $password = $credentials['password'];
        $user = self::userFromSpecifier($login);

        if (!$user || !Hash::check($password, $user->password)) {
            return null;
        }

        return $user;
    }
}
