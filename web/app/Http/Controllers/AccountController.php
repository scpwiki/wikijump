<?php

declare(strict_types=1);

namespace Wikijump\Http\Controllers;

use Illuminate\Contracts\Auth\StatefulGuard;
use Illuminate\Http\Response;

/**
 * Controller for handling account related requests.
 * API: `/account`
 */
class AccountController extends Controller
{
    /** Guard used to handle authentication. */
    private StatefulGuard $guard;

    /**
     * @param StatefulGuard $guard
     */
    public function __construct(StatefulGuard $guard)
    {
        $this->guard = $guard;
    }

    /**
     * Sends a verification email to the account's address.
     * Endpoint: `POST:/account/send-verification-email` | `accountSendVerificationEmail`
     */
    public function sendVerificationEmail(): Response
    {
        // check if user isn't logged in
        if (!$this->guard->check()) {
            return new Response('', 401);
        }

        /** @var \Wikijump\Models\User */
        $user = $this->guard->user();

        // unlikely edge case: account doesn't have an email address
        if (!$user->email) {
            return new Response('', 400);
        }

        // check if email has already been verified
        if ($user->hasVerifiedEmail()) {
            return new Response('', 403);
        }

        // send verification email
        $user->sendEmailVerificationNotification();

        return new Response('', 202);
    }
}
