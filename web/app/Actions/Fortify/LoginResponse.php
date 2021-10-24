<?php

declare(strict_types=1);

namespace Wikijump\Actions\Fortify;

use Illuminate\Http\Request;
use Laravel\Fortify\Contracts\LoginResponse as LoginResponseContract;
use Symfony\Component\HttpFoundation\Response;

/**
 * LoginResponse implementation to redirect user back to their requested page.
 *
 * @package Wikijump\Actions\Fortify
 */
class LoginResponse implements LoginResponseContract
{
    /**
     * Create an HTTP response that represents the object.
     *
     * @param  Request  $request
     * @return Response
     */
    public function toResponse($request): Response
    {
        if ($request->wantsJson()) {
            return response()->json(['two_factor' => false]);
        }

        $previousLocation = session('backUrl');
        return $previousLocation ? redirect()->to($previousLocation) : redirect()->home();
    }
}
