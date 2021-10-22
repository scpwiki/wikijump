<?php

declare(strict_types=1);

namespace Wikijump\Http\Controllers;

use Illuminate\Http\Response;
use Illuminate\Http\Request;
use Illuminate\Validation\ValidationException;
use Laravel\Fortify\Actions\ConfirmPassword;
use Laravel\Fortify\Http\Controllers\AuthenticatedSessionController;
use Wikijump\Http\Requests\LoginRequest;

/**
 * Controller for authenticating users.
 * API: `/auth`
 */
class AuthController extends AuthenticatedSessionController
{
    /**
     * Attempts a login. The login specifier can be either a username or an email address.
     * Endpoint: `POST:/auth/login` | `authLogin`
     * @param LoginRequest $request The request containing user credentials.
     */
    public function login(LoginRequest $request): Response
    {
        // check if the user is already logged in
        if ($this->guard->check()) {
            return new Response('', 409);
        }

        // attempts to logs the user in
        // Fortify returns its own response, but we want to return our own
        $response = $this->store($request);

        if ($response->status() === 200) {
            return new Response(['csrf' => $request->session()->token()], 200);
        } else {
            // TODO: more detailed errors (e.g. bad email)
            return new Response('', 400);
        }
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

        try {
            $request->validate(['password' => 'required|string']);
        } catch (ValidationException $err) {
            return new Response('', 400);
        }

        // what follows is what ConfirmablePasswordController does
        // internally, but we want to return our own response

        $confirmed = app(ConfirmPassword::class)(
            $this->guard,
            $request->user(),
            $request->input('password'),
        );

        if ($confirmed) {
            $request->session()->put('auth.password_confirmed_at', time());
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
            $this->destroy($request);
            return new Response('', 200);
        } else {
            // user isn't logged in, so we can't log them out
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
