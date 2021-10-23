<?php
declare(strict_types=1);

namespace Wikijump\Services\Wikitext;

class TextOutput
{
    /**
     * @var string The textual output the rendering process.
     *
     * This is a complete document, at least as much as that can be said of plain text.
     * The output has no formatting, or suggestions of formatting (e.g. surrounding bold items with "*").
     */
    public string $text;

    /**
     * @var array The list of ParseWarning objects, if any, generated during parsing.
     */
    public array $warnings;

    public function __construct(string $text, array $warnings)
    {
        $this->text = $text;
        $this->warnings = $warnings;
    }
}
