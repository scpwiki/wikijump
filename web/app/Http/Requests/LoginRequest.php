<?php

declare(strict_types=1);

namespace Wikijump\Http\Requests;

use Laravel\Fortify\Http\Requests\LoginRequest as FortifyLoginRequest;

/**
 * Request that satisfies Fortify's LoginRequest interface,
 * but without Fortify's hardcoded validation rules.
 */
class LoginRequest extends FortifyLoginRequest
{
    public function rules()
    {
        return [];
    }
}
