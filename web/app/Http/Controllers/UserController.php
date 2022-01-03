<?php

declare(strict_types=1);

namespace Wikijump\Http\Controllers;

use Exception;
use Illuminate\Contracts\Auth\StatefulGuard;
use Illuminate\Http\Request;
use Illuminate\Http\Response;
use Wikijump\Models\User;
use Wikijump\Services\Deepwell\DeepwellService;
use Wikijump\Services\Users\UserValidation;

/**
 * Controller for interacting with the user model.
 * API: `/users`
 */
class UserController extends Controller
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

    private function resolveUser(Request $request): ?User
    {
        $path_type = (string) $request->input('path_type');
        $path = (string) $request->input('path');

        $user = null;

        if ($path_type === 'slug') {
            $user = User::where('slug', $path)->first();
        } elseif ($path_type === 'id') {
            $user = User::find((int) $path);
        }

        return $user;
    }

    private function resolveClient(): ?User
    {
        if (!$this->guard->check()) {
            return null;
        }

        return $this->guard->user();
    }

    private function data(Request $request, User $user): object
    {
        $detail = (string) $request->query('detail', 'identity');
        $avatars = (bool) $request->query('avatars', true);

        $obj = DeepwellService::getInstance()->getUserById($user->id, $detail);

        if (!$avatars) {
            $obj->tinyavatar = null;
        }

        return $obj;
    }

    private function clientData(Request $request): ?object
    {
        $client = $this->resolveClient();

        if ($client === null) {
            return null;
        }

        return $this->data($request, $client);
    }

    private function userData(Request $request): ?object
    {
        $user = $this->resolveUser($request);

        if ($user === null) {
            return null;
        }

        return $this->data($request, $user);
    }

    private static function addFromRename(
        array &$target,
        array &$source,
        string $from,
        string $to
    ): void {
        if (array_key_exists($from, $source)) {
            $target[$to] = $source[$from];
        }
    }

    private static function addFromRenameAll(
        array &$target,
        array &$source,
        array $from_to
    ): void {
        foreach ($from_to as $from => $to) {
            self::addFromRename($target, $source, $from, $to);
        }
    }

    // -- CLIENT

    /**
     * Gets the client's user details.
     * API: `GET:/user` | `userClientGet`
     */
    public function clientGet(Request $request): Response
    {
        $obj = $this->clientData($request);

        if (!$obj) {
            return new Response('', 401);
        }

        return new Response(json_encode($obj), 200);
    }

    /**
     * Update (patch) the client's user details.
     * API: `PATCH:/user` | `userClientUpdateProfile`
     */
    public function clientUpdateProfile(Request $request): Response
    {
        $client = $this->resolveClient();

        if (!$client) {
            return new Response('', 401);
        }

        // TODO: signature, location, links
        $patch = $request->only(['about', 'realname', 'pronouns', 'birthday']);

        // we're gonna build this object up and send it to Deepwell
        $data = [];

        self::addFromRenameAll($data, $patch, [
            'about' => 'bio',
            'realname' => 'real_name',
            'pronouns' => 'pronouns',
            'birthday' => 'dob',
        ]);

        if (isset($data['bio'])) {
            // we're going to escape this for now,
            // but we should probably consider this as Wikitext
            // and have it get compiled on Deepwell's end
            // TODO: should we be paranoid about other strings?
            $data['bio'] = htmlspecialchars($data['bio']);
        }

        try {
            DeepwellService::getInstance()->setUser($client->id, $data);
        } catch (Exception $e) {
            return new Response('', 400);
        }

        return new Response('', 200);
    }

    /**
     * Gets the client's avatar.
     * This won't return the avatar directly, but rather return the URL for it.
     * API: `GET:/user/avatar` | `userGetClientAvatar`
     */
    public function clientGetAvatar(): Response
    {
        $client = $this->resolveClient();

        if (!$client) {
            return new Response('', 401);
        }

        return new Response(['avatar' => $client->avatar()], 200);
    }

    /**
     * Sets the client's avatar.
     * API: `PUT:/user/avatar` | `userSetClientAvatar`
     */
    public function clientSetAvatar(Request $request): Response
    {
        $client = $this->resolveClient();

        if (!$client) {
            return new Response('', 401);
        }

        // note: isn't documented that the file name is "avatar",
        // but it has to be named something specific to work I think
        $avatar = $request->file('avatar');

        if (!UserValidation::isValidAvatar($avatar)) {
            return new Response('', 400);
        }

        $client->updateAvatar($avatar);

        return new Response('', 200);
    }

    /**
     * Removes the client's avatar.
     * API: `DELETE:/user/avatar` | `userClientRemoveAvatar`
     */
    public function clientRemoveAvatar(): Response
    {
        $client = $this->resolveClient();

        if (!$client) {
            return new Response('', 401);
        }

        $client->deleteAvatar();

        return new Response('', 200);
    }

    // -- USER

    /**
     * Gets a user's details.
     * API: `GET:/user/{path_type}/{path}` | `userGet`
     */
    public function get(Request $request): Response
    {
        $obj = $this->userData($request);

        if (!$obj) {
            return new Response('', 404);
        }

        return new Response(json_encode($obj), 200);
    }

    /**
     * Gets a user's avatar.
     * This won't return the avatar directly, but rather return the URL for it.
     * API: `GET:/user/{path_type}/{path}/avatar` | `userGetAvatar`
     */
    public function getAvatar(Request $request): Response
    {
        $user = $this->resolveUser($request);

        if ($user === null) {
            return new Response('', 404);
        }

        return new Response(['avatar' => $user->avatar()], 200);
    }
}
