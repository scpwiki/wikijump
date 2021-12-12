<?php

declare(strict_types=1);

namespace Wikijump\Mail;

use Illuminate\Container\Container;
use Illuminate\Notifications\Messages\MailMessage;
use Wikijump\Services\MJML\MJML;

/**
 * MailMessage that supports using MJML, and text fallbacks.
 *
 * Use the `mjml` method to set the MJML view, and likewise for `text`.
 */
class MJMLMessage extends MailMessage
{
    /** MJML template path. */
    public ?string $mjml = 'emails.message.mjml';

    /** Text fallback template path. */
    public ?string $text = 'emails.message.text';

    /**
     * Constructs a new `MJMLMessage`.
     */
    public function __construct()
    {
        // MailMessage sets this automatically, which we don't want
        $this->markdown = null;
    }

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

    /**
     * Sets the text fallback template to use.
     *
     * @param string $text Path to the text template.
     * @param array $data Data to pass to the template.
     * @return $this
     */
    public function text(string $text, array $data = [])
    {
        $this->text = $text;
        $this->viewData = array_merge($this->viewData, $data);

        return $this;
    }

    /**
     * Renders the notification message into an HTML string.
     */
    public function render(): string
    {
        // I'm really not quite sure how this all works
        // honestly the way Laravel has this set up is bizarre
        // and really hard to follow

        if (isset($this->mjml)) {
            if (isset($this->text)) {
                $data = $this->data();

                $html = MJML::render($this->mjml, $data);
                $text = $this->text;

                /** @var \Illuminate\Mail\Mailer $mailer */
                $mailer = Container::getInstance()->make('mailer');

                // it's important that the `html` property is Htmlable
                // and that the `text` property is just a template identifier

                return $mailer->render(['html' => $html, 'text' => $text], $data);
            } else {
                return MJML::render($this->mjml, $this->data())->toHtml();
            }
        }

        // normal MailMessage supports text fallback, but differently.
        // it makes the $view property an array, so before we go back to
        // the parent method we need to make sure $view is setup correctly

        if (isset($this->view, $this->text) && !is_array($this->view)) {
            $this->view = [$this->view, $this->text];
        }

        return parent::render();
    }
}
