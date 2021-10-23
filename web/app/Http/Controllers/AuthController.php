<?php

declare(strict_types=1);

namespace Wikijump\Http\Controllers;

use Illuminate\Contracts\Auth\StatefulGuard;
use Illuminate\Http\Response;
use Illuminate\Http\Request;
use Wikijump\Services\Authentication\Authentication;
use Wikijump\Services\Authentication\AuthenticationError;

/**
 * Controller for authenticating users.
 * API: `/auth`
 */
class AuthController extends Controller
{
    /** Guard used to handle authentication. */
    private StatefulGuard $guard;

    /** @param StatefulGuard $guard */
    public function __construct(StatefulGuard $guard)
    {
        $this->guard = $guard;
    }

    /**
     * Attempts a login. The login specifier can be either a username or an email address.
     * Endpoint: `POST:/auth/login` | `authLogin`
     * @param Request $request The request containing user credentials.
     */
    public function login(Request $request): Response
    {
        // check if the user is already logged in
        if ($this->guard->check()) {
            return new Response('', 409);
        }

        // atempt to get the user for the given credentials
        $result = Authentication::authenticate($request);

        if (!$result->ok()) {
            $error = $result->error();
            // TODO: specific response codes for different error types
            switch ($error) {
                case AuthenticationError::FAILED_TO_VALIDATE:
                    return new Response('', 400);
                case AuthenticationError::INVALID_PASSWORD:
                    return new Response('', 400);
                case AuthenticationError::INVALID_SPECIFIER:
                    return new Response('', 400);
                default:
                    return new Response('', 400);
            }
        }

        $user = $result->user();
        $remember = $request->input('remember', false);

        $this->guard->login($user, $remember);

        // prevent session fixation
        $request->session()->regenerate();

        // return new CSRF due to regenerated session
        return new Response(['csrf' => $request->session()->token()], 200);
    }

    /**
     * Confirms the client's password.
     * Endpoint: `POST:/auth/confirm` | `authConfirm`
     * @param Request $request The request containing the password.
     */
    public function confirm(Request $request): Response
    {
        // can't confirm a password if the user is not logged in
        if (!$this->guard->check()) {
            return new Response('', 401);
        }

        $confirmed = $this->guard->validate([
            'username' => $request->user()->username,
            'password' => $request->input('password'),
        ]);

        if ($confirmed) {
            $request->session()->passwordConfirmed();
            return new Response('', 200);
        }

        return new Response('', 400);
    }

    /**
     * Logs the client out.
     * Endpoint: (authed) `DELETE:/auth/logout` | `authLogout`
     * @param Request $request The current request.
     */
    public function logout(Request $request): Response
    {
        if ($this->guard->check()) {
            $this->guard->logout();
            $request->session()->invalidate();
            $request->session()->regenerateToken();

            return new Response('', 200);
        }
        // user isn't logged in, so we can't log them out
        else {
            return new Response('', 401);
        }
    }

    /**
     * Gets the authentication state of the client.
     * Endpoint: `POST:/auth/check` | `authCheck`
     * @param Request $request The current request.
     */
    public function check(Request $request): Response
    {
        $session_valid = $request->session()->isStarted();
        $authed = $this->guard->check();
        return new Response(['sessionValid' => $session_valid, 'authed' => $authed], 200);
    }

    /**
     * Refreshes the client's session.
     * Endpoint: (authed) `POST:/auth/refresh` | `authRefresh`
     * @param Request $request The current request.
     */
    public function refresh(Request $request): Response
    {
        // check if session is invalid
        if (!$request->session()->isStarted()) {
            return new Response('', 403);
        }
        // check if the user is logged in
        elseif ($this->guard->check()) {
            $request->session()->regenerate();
            return new Response(['csrf' => $request->session()->token()], 200);
        }
        // user is logged out, so we can't refresh
        else {
            return new Response('', 401);
        }
    }
}
