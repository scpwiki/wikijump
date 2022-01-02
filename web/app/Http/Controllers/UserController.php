<?php

declare(strict_types=1);

namespace Wikijump\Http\Controllers;

use Illuminate\Contracts\Auth\StatefulGuard;
use Illuminate\Http\Request;
use Illuminate\Http\Response;
use Wikijump\Models\User;
use Wikijump\Services\Deepwell\DeepwellService;

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
