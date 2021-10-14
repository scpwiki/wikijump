<?php
declare(strict_types=1);

namespace Wikijump\Services\Wikitext;

class DummyBackend extends WikitextBackend
{
    public function renderHtml(string $wikitext): HtmlOutput
    {
        $html = '<span class="output">(Rendered HTML here)</span>';
        $style = '.output { color: forestgreen; }';
        $backlinks = new Backlinks([], [], [], [], []);

        return new HtmlOutput($html, $style, [], [], $backlinks);
    }

    public function renderText(string $wikitext): TextOutput
    {
        return new TextOutput('(Rendered text here)', []);
    }

    public function version(): string
    {
        return 'dummy 0.0.0';
    }
}
