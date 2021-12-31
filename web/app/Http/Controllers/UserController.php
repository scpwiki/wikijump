<?php

declare(strict_types=1);

namespace Wikijump\Http\Controllers;

use Illuminate\Contracts\Auth\StatefulGuard;
use Illuminate\Http\Request;
use Illuminate\Http\Response;
use Illuminate\Support\Facades\Log;
use Wikijump\Models\User;

/**
 * Controller for interacting with the user model.
 * API: `/users`
 */
class UserController extends Controller
{
    const DEFAULT_AVATAR_URL = '/files--static/media/default-avatar.png';

    /** Guard used to handle authentication. */
    private StatefulGuard $guard;

    /**
     * @param StatefulGuard $guard
     */
    public function __construct(StatefulGuard $guard)
    {
        $this->guard = $guard;
    }

    public function getAvatar(Request $request): Response
    {
        $path_type = $request->input('path_type');
        $path = $request->input('path');

        $user = null;

        if ($path_type === 'name') {
            $user = User::where('slug', $path)->first();
        } elseif ($path_type === 'id') {
            $user = User::find($path);
        }

        if ($user === null) {
            return new Response('', 404);
        }

        $avatar = $user->avatar();

        if ($avatar === null) {
            return new Response(['avatar' => self::DEFAULT_AVATAR_URL], 200);
        }

        return new Response(['avatar' => $avatar], 200);
    }

    public function clientGetAvatar(): Response
    {
        if (!$this->guard->check()) {
            return new Response('', 401);
        }

        /** @var User */
        $user = $this->guard->user();

        $avatar = $user->avatar();

        if ($avatar === null) {
            return new Response(['avatar' => self::DEFAULT_AVATAR_URL] . 200);
        }

        Log::debug($avatar);

        return new Response(['avatar' => $avatar], 200);
    }
}
