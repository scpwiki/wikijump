<?php

declare(strict_types=1);

namespace Wikijump\Http\Controllers;

use Exception;
use Illuminate\Auth\Events\PasswordReset;
use Illuminate\Contracts\Auth\StatefulGuard;
use Illuminate\Http\Request;
use Illuminate\Http\Response;
use Illuminate\Support\Facades\Hash;
use Illuminate\Support\Facades\Password;
use Illuminate\Support\Str;
use Wikidot\Utils\WDStringUtils;
use Wikijump\Models\User;
use Wikijump\Services\Deepwell\DeepwellService;
use Wikijump\Services\Users\UserValidation;

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

    private function resolveClient(): ?User
    {
        if (!$this->guard->check()) {
            return null;
        }

        return $this->guard->user();
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
     * Starts password recovery.
     * Endpoint: `POST:/account/start-recovery` | `accountStartRecovery`
     */
    public function startRecovery(Request $request): Response
    {
        // check if user is logged in
        if ($this->guard->check()) {
            return new Response('', 409);
        }

        $email = $request->input('email');

        // no one is using this email, return 403
        if (!UserValidation::isEmailTaken($email)) {
            return new Response('', 403);
        }

        // send recovery email

        $status = Password::sendResetLink($request->only('email'));

        switch ($status) {
            case Password::RESET_LINK_SENT:
                return new Response('', 202);

            case Password::INVALID_USER:
                return new Response('', 403);

            default:
                return new Response('', 500);
        }
    }

    /**
     * Gets the current email address.
     * Endpoint: `GET:/account/email` | `accountGetEmail`
     */
    public function getEmail(): Response
    {
        $client = $this->resolveClient();

        if (!$client) {
            return new Response('', 401);
        }

        return new Response(['email' => $client->email], 200);
    }

    /**
     * Sends a verification email to the account's address.
     * Endpoint: `POST:/account/send-verification-email` | `accountSendVerificationEmail`
     */
    public function sendVerificationEmail(): Response
    {
        $client = $this->resolveClient();

        if (!$client) {
            return new Response('', 401);
        }

        // unlikely edge case: account doesn't have an email address
        if (!$client->email) {
            return new Response('', 400);
        }

        // check if email has already been verified
        if ($client->hasVerifiedEmail()) {
            return new Response('', 403);
        }

        // send verification email
        $client->sendEmailVerificationNotification();

        return new Response('', 202);
    }

    /**
     * Gets the current account settings.
     * Endpoint: `GET:/account/settings` | `accountGetSettings`
     */
    public function getSettings(): Response
    {
        $client = $this->resolveClient();

        if (!$client) {
            return new Response('', 401);
        }

        $accepts_invites = (bool) $client->get('receive_invitations');
        $language = (string) $client->language ?? '';
        $allow_messages = (string) $client->get('receive_pm');

        // this is a complete guess
        // values for `receive_pm` are:
        // - 'a'  (assumption: 'registered')
        // - 'mf' (assumption: 'co-members' & contacts)
        // - 'f'  (assumption: contacts)
        // - 'n'  (assumption: 'nobody')
        // we don't handle contacts, so we just assume 'co-members'

        // prettier-ignore
        switch ($allow_messages) {
            case 'a':  $allow_messages = 'registered'; break;
            case 'mf': $allow_messages = 'co-members'; break;
            case 'f':  $allow_messages = 'co-members'; break;
            case 'n':  $allow_messages = 'nobody';     break;
        }

        $settings = [
            'acceptsInvites' => $accepts_invites,
            'language' => $language,
            'allowMessages' => $allow_messages,
        ];

        return new Response($settings, 200);
    }

    /**
     * Update (patch) the client's user details.
     * Endpoint: `POST:/account/settings` | `accountUpdateSettings`
     */
    public function updateSettings(Request $request): Response
    {
        $client = $this->resolveClient();

        if (!$client) {
            return new Response('', 401);
        }

        if ($request->has('language')) {
            $language = (string) $request->input('language');
            try {
                // we'll update via deepwell because it can properly handle
                // language code validation
                DeepwellService::getInstance()->setUser($client->id, [
                    'language' => $language,
                ]);
            } catch (Exception $e) {
                // we'll presume a bad language code was given
                return new Response('', 400);
            }
        }

        if ($request->has('acceptsInvites')) {
            $accepts_invites = (bool) $request->input('acceptsInvites');
            $client->set(['receive_invitations' => $accepts_invites]);
        }

        if ($request->has('allowMessages')) {
            $allow_messages = (string) $request->input('allowMessages');
            // parse based on our previous guesses
            // prettier-ignore
            switch ($allow_messages) {
                case 'registered':  $allow_messages = 'a';  break;
                case 'co-members':  $allow_messages = 'mf'; break;
                case 'nobody':      $allow_messages = 'n';  break;
                default:            $allow_messages = null; break;
            }

            if (!$allow_messages) {
                return new Response('', 400);
            }

            $client->set(['receive_pm' => $allow_messages]);
        }

        return new Response('', 200);
    }

    /**
     * Handles updating a password during password recovery.
     * Not part of the "proper" API.
     */
    public function handlePasswordRecoveryUpdate(Request $request): Response
    {
        $request->validate([
            'token' => 'required',
            'email' => 'required|email',
            'password' => 'required|string',
        ]);

        // do our more strict check on the password
        if (!UserValidation::isValidPassword($request->input('password'))) {
            return new Response('', 400);
        }

        $credentials = $request->only(['email', 'password', 'token']);

        $status = Password::reset($credentials, function ($user, $password) {
            $user
                ->forceFill(['password' => Hash::make($password)])
                ->setRememberToken(Str::random(60));

            $user->save();

            event(new PasswordReset($user));
        });

        switch ($status) {
            case Password::PASSWORD_RESET:
                return new Response('', 202);

            default:
                return new Response('', 500);
        }
    }
}
