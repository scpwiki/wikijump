<?php
declare(strict_types = 1);

namespace Wikijump\Services\Wikitext;

class NullBackend implements WikitextBackend
{
    public function renderHtml(string $wikitext): HtmlOutput {
        return new HtmlOutput('', '', [], []);
    }

    public function renderText(string $wikitext): TextOutput {
        return new TextOutput('', []);
    }

    public function version(): string {
        return 'null 0.0.0';
    }
}
