<?php

declare(strict_types=1);

namespace Wikijump\Actions\Fortify;

use Illuminate\Validation\ValidationException;
use Wikidot\Utils\WDStringUtils;
use Wikijump\Models\User;
use Illuminate\Support\Facades\Hash;
use Illuminate\Support\Facades\Validator;
use Laravel\Fortify\Contracts\CreatesNewUsers;
use Laravel\Jetstream\Jetstream;

/**
 * Fortify method for creating new User accounts.
 * @package Wikijump\Actions\Fortify
 */
class CreateNewUser implements CreatesNewUsers
{
    use PasswordValidationRules;

    /**
     * Validate and create a newly registered user.
     *
     * @param array $input
     * @return User
     * @throws ValidationException
     */
    public function create(array $input): User
    {
        $input['unix_name'] = WDStringUtils::toUnixName($input['username']);

        Validator::make($input, [
            'username' => ['required', 'string', 'max:255', 'unique:users'],
            'unix_name' => ['unique:users'],
            'email' => ['required', 'string', 'email', 'max:255', 'unique:users'],
            'password' => $this->passwordRules(),
            'terms' => Jetstream::hasTermsAndPrivacyPolicyFeature() ? ['required', 'accepted'] : '',
        ])->validate();

        return (new User())->create([
            'username' => $input['username'],
            'unix_name' => $input['unix_name'],
            'email' => $input['email'],
            'password' => Hash::make($input['password']),
        ]);
    }
}
