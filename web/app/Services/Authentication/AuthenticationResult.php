<?php

declare(strict_types=1);

namespace Wikijump\Services\Authentication;

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
    const FAILED_TO_VALIDATE = 'failed_to_validate';
    const INVALID_PASSWORD = 'invalid_password';
    const INVALID_SPECIFIER = 'unknown_specifier';
}

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

    public function ok(): bool
    {
        return $this->user !== null && $this->error === null;
    }

    public function user(): User
    {
        if (!$this->ok()) {
            throw new LogicException('User is null');
        }

        return $this->user;
    }

    public function error(): string
    {
        if ($this->ok()) {
            throw new LogicException('Error is null');
        }

        return $this->failure;
    }
}
