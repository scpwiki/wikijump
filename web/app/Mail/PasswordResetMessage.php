<?php

declare(strict_types=1);

namespace Wikijump\Mail;

/**
 * Generic password reset message.
 */
class PasswordResetMessage extends MJMLMessage
{
    /**
     * Constructs a new password reset message.
     *
     * @param string $url The URL to the password reset page.
     * @param int $expires The number of minutes until the password reset expires.
     */
    public function __construct(string $url, int $expires)
    {
        parent::__construct();

        $this->subject(__('email.reset_password.SUBJECT'))
            ->greeting(__('email.reset_password.GREETING'))
            ->line(__('email.reset_password.INTRO'))
            ->action(__('email.reset_password.ACTION'), $url)
            ->line(__('email.reset_password.EXPIRES', ['count' => $expires]))
            ->line(__('email.reset_password.OUTRO'));
    }
}
