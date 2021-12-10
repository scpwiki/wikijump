<?php

declare(strict_types=1);

namespace Wikijump\Mail;

use Illuminate\Mail\Mailable;
use Wikijump\Services\MJML\MJML;

/** Mailable that uses MJML-based Blade templates. */
class MJMLMailable extends Mailable
{
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
                'text' => $this->textView ?? null,
            ]);
        }

        $data = $this->buildViewData();

        $html = null;
        $text = null;

        // render HTML, text, and/or markdown.

        if (isset($this->view) || isset($this->markdown)) {
            // prioritize MJML view over markdown
            if (isset($this->view)) {
                $html = MJML::render($this->view, $data);
            } else {
                $built = $this->buildMarkdownView();
                $html = $built['html'];
                $text = $built['text'];
            }
        }

        // setting a plaintext view will override the markdown plaintext
        // this differs from default behavior of Mailable, but I feel this is
        // better than having markdown override an explicitly given plaintext
        if (isset($this->textView)) {
            $text = $this->textView;
        }

        // return output

        if ($html || $text) {
            return array_filter([
                'html' => $html,
                'text' => $text,
            ]);
        }

        // apparently this is what Mailable does as its final fallback
        return $this->view;
    }
}
