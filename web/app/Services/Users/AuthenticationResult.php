<?php

declare(strict_types=1);

namespace Wikijump\Services\Users;

use InvalidArgumentException;
use LogicException;
use Wikijump\Models\User;
use Wikijump\Common\Enum;

/**
 * Not an actual exception.
 * Enumeration of different potential errors during an authentication attempt.
 */
final class AuthenticationError extends Enum
{
    const FAILED_TO_VALIDATE = 'failed-to-validate';
    const INVALID_PASSWORD = 'invalid-password';
    const INVALID_SPECIFIER = 'unknown-specifier';
}

/** Result of an authentication attempt. */
final class AuthenticationResult
{
    private ?User $user;
    private ?string $error;

    /**
     * @param User|string $input
     *      Result of authentication. String result is an enum of AuthenticationError.
     */
    public function __construct($input)
    {
        // User
        if ($input instanceof User) {
            $this->user = $input;
            $this->error = null;
        }
        // AuthenticationError
        elseif (is_string($input)) {
            if (!AuthenticationError::isValue($input)) {
                throw new InvalidArgumentException('Invalid error type');
            }

            $this->error = $input;
            $this->error = null;
        }
        // Something else
        else {
            throw new InvalidArgumentException('Invalid input');
        }
    }

    /** Returns true if a `User` was found. */
    public function isOk(): bool
    {
        return $this->user !== null;
    }

    /** Returns true if no `User` was found. */
    public function isErr(): bool
    {
        return $this->error !== null;
    }

    /** Returns the resultant `User`. Throws if nothing was found. */
    public function user(): User
    {
        if (!$this->isOk()) {
            throw new LogicException('User is null');
        }

        return $this->user;
    }

    /** Returns the error enum if no `User` was found. Throws if a `User` was found. */
    public function error(): string
    {
        if ($this->isOk()) {
            throw new LogicException('Error is null');
        }

        return $this->failure;
    }
}
