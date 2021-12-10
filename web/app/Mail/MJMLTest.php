<?php

declare(strict_types=1);

namespace Wikijump\Mail;

use Exception;
use Illuminate\Bus\Queueable;
use Illuminate\Queue\SerializesModels;

class MJMLTest extends MJMLMailable
{
    use Queueable, SerializesModels;

    /** Rendering mode. */
    private ?string $mode;

    /**
     * Creates a new message instance.
     *
     * @param string $mode Sets the rendering mode.
     */
    public function __construct(?string $mode = null)
    {
        $this->mode = $mode;
    }

    /**
     * Builds the message.
     */
    public function build(): self
    {
        if (empty($this->mode)) {
            return $this->mjml('emails.test.mjml')
                ->view('emails.test.view')
                ->markdown('emails.test.markdown')
                ->text('emails.test.text');
        }

        // prettier-ignore
        switch ($this->mode) {
            case 'mjml':     return $this->mjml('emails.test.mjml');
            case 'view':     return $this->view('emails.test.view');
            case 'markdown': return $this->markdown('emails.test.markdown');
            case 'text':     return $this->text('emails.test.text');
            default:         throw new Exception('Unknown mode');
        }
    }
}
