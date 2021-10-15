<?php

declare(strict_types=1);

namespace Wikijump\Http\Controllers;

use Illuminate\Contracts\Routing\ResponseFactory;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\Request;

/**
 * Controller for authenticating users.
 * API: /auth
 */
class AuthController extends Controller
{
    /**
     * Attempts a login. The login specifier can be either a username or an email address.
     * Endpoint: `POST` | `/auth/login` | `authLogin`
     * @param Request $request The request containing user credentials.
     */
    public function login(Request $request): ResponseFactory
    {
        // check if the user is already logged in
        if (Auth::check()) {
            response('', 409);
        }

        $credentials = $request->validate([
            'login' => 'required',
            'password' => 'required',
            'remember' => 'sometimes|boolean',
        ]);

        $login = $credentials['login'];
        $password = $credentials['password'];
        $remember = $credentials['remember'] ?? false;
        $isEmail = filter_var($login, FILTER_VALIDATE_EMAIL);

        $attempt = $isEmail
            ? Auth::attempt(['email' => $login, 'password' => $password], $remember)
            : Auth::attempt(['username' => $login, 'password' => $password], $remember);

        if ($attempt) {
            $request->session()->regenerate();
            return response('', 200);
        }

        return response('', 400);
    }

    /**
     * Logs the client out.
     * Endpoint: (authed) `DELETE` | `/auth/logout` | `authLogout`
     * @param Request $request The current request.
     */
    public function logout(Request $request): ResponseFactory
    {
        if (Auth::check()) {
            Auth::logout();
            $request->session()->invalidate();
            $request->session()->regenerateToken();
            return response('', 200);
        } else {
            // user isn't logged in, so we can't log them out
            return response('', 400);
        }
    }

    /**
     * Gets the authentication state of the client.
     * Endpoint: `POST` | `/auth/check` | `authCheck`
     * @param Request $request The current request.
     */
    public function check(Request $request): ResponseFactory
    {
        $session_valid = $request->session()->isStarted();
        $authed = Auth::check();
        return response(['session_valid' => $session_valid, 'authed' => $authed], 200);
    }

    /**
     * Refreshes the client's session.
     * Endpoint: (authed) `POST` | `/auth/refresh` | `authRefresh`
     * @param Request $request The current request.
     */
    public function refresh(Request $request): ResponseFactory
    {
        if (Auth::check()) {
            $request->session()->regenerate();
            return response('', 200);
        } else {
            return response('', 400);
        }
    }
}
