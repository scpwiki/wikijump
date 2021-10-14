<?php
declare(strict_types=1);

namespace Wikijump\Http\Middleware;

use Illuminate\Foundation\Http\Middleware\VerifyCsrfToken as Middleware;

class VerifyCsrfToken extends Middleware
{
    /**
     * The URIs that should be excluded from CSRF verification.
     *
     * @var array
     */
    protected $except = [
        '/ajax--handler', // TODO: Add the csrf token to requests to ajax--handler.
    ];
}
