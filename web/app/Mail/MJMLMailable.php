<?php

declare(strict_types=1);

namespace Wikijump\Mail;

use Illuminate\Mail\Mailable;
use Wikijump\Services\MJML\MJML;

/**
 * Mailable that supports using MJML.
 *
 * Use the `mjml` method to set the MJML content.
 */
class MJMLMailable extends Mailable
{
    /** MJML template path. */
    protected string $mjml;

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
     * Builds the view data for the message.
     *
     * @return array|string
     */
    protected function buildView()
    {
        // the html may have been manually set,
        // so we need to leave early and not render anything
        if ($this->html) {
            return array_filter([
                'html' => $this->html,
                'text' => $this->buildText(),
            ]);
        }

        if (isset($this->mjml)) {
            return array_filter([
                'html' => MJML::render($this->mjml, $this->buildViewData()),
                'text' => $this->buildText(),
            ]);
        }

        // default mailable behavior follows

        if (isset($this->markdown)) {
            return $this->buildMarkdownView();
        }

        if (isset($this->view, $this->textView)) {
            return [$this->view, $this->textView];
        } elseif (isset($this->textView)) {
            return ['text' => $this->textView];
        }

        return $this->view;
    }

    /**
     * Returns the text view for the message.
     */
    protected function buildText(): ?string
    {
        if (isset($this->textView)) {
            return $this->textView;
        }

        if (isset($this->markdown)) {
            return $this->buildMarkdownView()['text'];
        }

        return null;
    }
}
