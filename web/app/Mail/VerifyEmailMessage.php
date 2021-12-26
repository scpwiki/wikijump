<?php

declare(strict_types=1);

namespace Wikijump\Mail;

/**
 * Generic message for email verification.
 */
class VerifyEmailMessage extends MJMLMessage
{
    /**
     * Constructs a new email verification message.
     *
     * @param string $url The url to the email verification page.
     */
    public function __construct(string $url)
    {
        parent::__construct();

        $this->subject(__('emails-verify-email.subject'))
            ->greeting(__('emails-verify-email.greeting'))
            ->line(__('emails-verify-email.intro'))
            ->action(__('emails-verify-email.action'), $url)
            ->line(__('emails-verify-email.outro'));
    }
}
