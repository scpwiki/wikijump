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
        $has_view = isset($this->view);
        $has_text = isset($this->textView);
        $data = $this->viewData;

        $html = '';
        $text = '';

        if ($has_view) {
            $html = MJML::render($this->view, $data);
        }

        if ($has_text) {
            $text = view($this->textView, $data)->render();
        }

        if ($has_view && $has_text) {
            return [
                'html' => $html,
                'text' => $text,
            ];
        } elseif ($has_view) {
            return $html;
        } elseif ($has_text) {
            return $text;
        }

        return '';
    }
}
