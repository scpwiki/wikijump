<?php

declare(strict_types=1);

namespace Wikijump\Mail;

use Illuminate\Notifications\Messages\MailMessage;
use Wikijump\Services\MJML\MJML;

/**
 * MailMessage that supports using MJML.
 *
 * Use the `mjml` method to set the MJML view.
 */
class MJMLMessage extends MailMessage
{
    /** MJML template path. */
    protected string $mjml = 'emails.message.mjml';

    /**
     * Sets the MJML template to use. This template has the highest priority.
     * Templates will be rendered by Blade first, and then by MJML.
     *
     * @param string $mjml Path to the MJML template, in Blade template syntax.
     * @param array $data Data to pass to the template.
     * @return $this
     */
    public function mjml(string $mjml, array $data = [])
    {
        $this->mjml = $mjml;
        $this->viewData = array_merge($this->viewData, $data);

        return $this;
    }

    public function render(): string
    {
        if (isset($this->mjml)) {
            return MJML::render($this->mjml, $this->data())->toHtml();
        }

        return parent::render();
    }
}
