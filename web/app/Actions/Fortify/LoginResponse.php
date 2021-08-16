<?php

namespace Wikijump\Actions\Fortify;

use Laravel\Fortify\Contracts\LoginResponse as LoginResponseContract;

class LoginResponse implements LoginResponseContract
{
    /**
     * Create an HTTP response that represents the object.
     *
     * @param  \Illuminate\Http\Request  $request
     * @return \Symfony\Component\HttpFoundation\Response
     */
    public function toResponse($request)
    {
        if($request->wantsJson())
        {
            return response()->json(['two_factor' => false]);
        }
        return session('backUrl') ?
            redirect()->to(session('backUrl'))
            : redirect()->route('dashboard');
    }
}
