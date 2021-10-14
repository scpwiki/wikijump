<?php

declare(strict_types=1);

namespace Wikijump\Actions\Fortify;

use Illuminate\Validation\Rules\Password;

/**
 * Define password rules for new accounts.
 * @package Wikijump\Actions\Fortify
 */
trait PasswordValidationRules
{
    /**
     * Get the validation rules used to validate passwords.
     * @see https://laravel.com/docs/8.x/validation#validating-passwords
     *
     * @return array
     */
    protected function passwordRules(): array
    {
        return ['required', 'string', Password::min(10)->uncompromised(10), 'confirmed'];
    }
}
