<?php
declare(strict_types = 1);

namespace Wikijump\Services\Wikitext;

final class HtmlUtilities
{
    private function __construct() {}

    public static function purify(string $html): string
    {
        $tidyConfig = [
            // We don't want the <html> or <head>, just the cleaned-up HTML body
            'show-body-only' => true,

            // Disable long-line wrapping
            'wrap' => 0,
        ];

        return tidy_repair_string($html, $tidyConfig);
    }
}
