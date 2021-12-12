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
        $this->subject(__('email.verify_email.SUBJECT'))
            ->greeting(__('email.verify_email.GREETING'))
            ->line(__('email.verify_email.INTRO'))
            ->action(__('email.verify_email.ACTION'), $url)
            ->line(__('email.verify_email.OUTRO'));
    }
}
