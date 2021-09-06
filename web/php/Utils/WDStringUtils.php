<?php

namespace Wikidot\Utils;

use \intl\Normalizer;

class WDStringUtils
{
    /**
     * This method normalizes a string, removing punctuation and other characters.
     * See https://scuttle.atlassian.net/wiki/spaces/WD/pages/541655041/Wikidot+Normal+Form
     * For another implementation, see https://github.com/scpwiki/wikidot-normalize
     *
     * @param string $text The text to be normalized.
     * @return string
     */
    public static function toUnixName(string $text): string
    {
        $text = trim($text);

        // Perform unicode normalization then case folding.
        $text = Normalizer::normalize($text, Normalizer::FORM_KC);
        $text = mb_convert_case($text, MB_CASE_FOLD, 'UTF-8');

        // Normalize string
        $text = preg_replace('/[^a-z0-9\-:_]/', '-', $text); // Replace non-alphanumeric characters
        $text = preg_replace('/^_/', ':_', $text); // Allow leading underscores (e.g. _default, _template)
        $text = preg_replace('/(?<!:)_/', '-', $text); // Clobber all other underscores
        $text = preg_replace('/^\-*/', '', $text); // Strip leading dashes
        $text = preg_replace('/\-*$/', '', $text); // Strip trailing dashes
        $text = preg_replace('/[\-]{2,}/', '-', $text); // Combine multiple dashes
        $text = preg_replace('/[:]{2,}/', ':', $text); // Combine multiple colons

        // Clean up boundaries
        $text = str_replace(':-', ':', $text);
        $text = str_replace('-:', ':', $text);
        $text = str_replace('_-', '_', $text);
        $text = str_replace('-_', '_', $text);

        // Clean up categories
        $text = preg_replace('/^:/', '', $text);
        $text = preg_replace('/:$/', '', $text);

        return $text;
    }

    public static function addTrailingNewline(string $text): string
    {
        if (!preg_match("/\n$/", $text)) {
            $text .= "\n";
        }

        return $text;
    }
}
