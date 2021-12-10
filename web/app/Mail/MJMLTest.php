<?php

declare(strict_types=1);

namespace Wikijump\Mail;

use Illuminate\Bus\Queueable;
use Illuminate\Queue\SerializesModels;

class MJMLTest extends MJMLMailable
{
    use Queueable, SerializesModels;

    /**
     * Builds the message.
     */
    public function build(): self
    {
        return $this->mjml('emails.test.mjml')
            ->view('emails.test.view')
            ->text('emails.test.text');
    }
}
