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

        $this->subject(__('emails-reset-password.subject'))
            ->greeting(__('emails-reset-password.greeting'))
            ->line(__('emails-reset-password.intro'))
            ->action(__('emails-reset-password.action'), $url)
            ->line(__('emails-reset-password.expires', ['count' => $expires]))
            ->line(__('emails-reset-password.outro'));
    }
}
