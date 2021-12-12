<?php

declare(strict_types=1);

namespace Wikijump\Http\Controllers;

use Illuminate\Contracts\Auth\StatefulGuard;
use Illuminate\Http\Request;
use Illuminate\Http\Response;
use Illuminate\Support\Facades\Hash;
use Wikidot\Utils\WDStringUtils;
use Wikijump\Models\User;
use Wikijump\Services\UserValidation\UserValidation;

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
     * Registers an account. Email validation will be required.
     * Endpoint: `POST:/account/register` | `accountRegister`
     */
    public function register(Request $request): Response
    {
        // check if user is already logged in
        if ($this->guard->check()) {
            return new Response('', 409);
        }

        // validate request

        $email = $request->input('email');
        $username = $request->input('username');
        $password = $request->input('password');

        // TODO: differentiate response codes for different validation errors
        if (
            !UserValidation::isValidEmail($email) ||
            !UserValidation::isValidUsername($username) ||
            !UserValidation::isValidPassword($password)
        ) {
            return new Response('', 403);
        }

        // slugify username - the isValidUsername method already checked if
        // the slug is unique so we're safe to do this
        $slug = WDStringUtils::toUnixName($username);

        // request validated: create user, send email, login, and return response

        /** @var User */
        $user = User::create([
            'username' => $username,
            'slug' => $slug,
            'email' => $email,
            'password' => Hash::make($password),
        ]);

        // send verification email
        $user->sendEmailVerificationNotification();

        // registering automatically logs in the user
        // this is needed so that the resending of the verification email works
        $this->guard->login($user);

        // prevent session fixation
        $request->session()->regenerate();

        // return new CSRF due to regenerated session
        return new Response(['csrf' => $request->session()->token()], 202);
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
