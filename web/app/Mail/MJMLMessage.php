<?php

declare(strict_types=1);

namespace Wikijump\Mail;

use Closure;
use Illuminate\Contracts\Support\Htmlable;
use Illuminate\Notifications\Messages\MailMessage;
use Wikijump\Services\MJML\MJML;

/**
 * MailMessage that supports using MJML, and text fallbacks.
 *
 * Use the `mjml` method to set the MJML view, and likewise for `text`.
 *
 * Make sure to call `parent::__construct()` in your constructor.
 */
class MJMLMessage extends MailMessage
{
    /** MJML template path. */
    public ?string $mjml = 'emails.message.mjml';

    /** Text fallback template path. */
    public ?string $text = 'emails.message.text';

    public $markdown = null;

    public function __construct()
    {
        $this->view = $this->makeViewArray();
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

        $this->view = $this->makeViewArray();
        $this->markdown = null;

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

        $this->view = $this->makeViewArray();

        return $this;
    }

    private function makeViewArray(): array
    {
        if (isset($this->mjml)) {
            // closures can't use the `$this` keyword, so we need to pass it in
            $self = $this;
            $dataCallback = function () use (&$self) {
                return $self->data();
            };

            return array_filter([
                'html' => new MJMLDeferredHtmlable($this->mjml, $dataCallback),
                'text' => $this->text,
            ]);
        }

        if (is_array($this->view)) {
            return $this->view;
        }

        return array_filter([$this->view, $this->text]);
    }
}

/**
 * MJML `Htmlable` that defers rendering until the `toHtml` method is called.
 * This allows us to render the MJML template when we're confident that the
 * view data is finalized. This is important because Laravel's mailer tries to
 * handle rendering itself. The only way to avoid that is to use a `Htmlable`
 * object, which the mailer will call `toHtml` on rather than rendering with Blade.
 */
class MJMLDeferredHtmlable implements Htmlable
{
    private string $view;

    private Closure $dataCallback;

    public function __construct(string $view, Closure $dataCallback)
    {
        $this->view = $view;
        $this->dataCallback = $dataCallback;
    }

    public function toHtml(): string
    {
        $data = ($this->dataCallback)();

        return MJML::render($this->view, $data)->toHtml();
    }
}
