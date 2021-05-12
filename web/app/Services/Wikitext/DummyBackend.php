<?php
declare(strict_types = 1);

namespace Wikijump\Services\Wikitext;

class DummyBackend implements WikitextBackend
{
    public function renderHtml(string $wikitext): HtmlOutput
    {
        return new HtmlOutput('<span class="output">(Rendered HTML here)</span>', '.output { color: green; }', [], []);
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
