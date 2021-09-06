<?php

namespace Wikidot\Utils;

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

        $text = mb_convert_case($text, MB_CASE_FOLD, 'UTF-8');
        $text = preg_replace('/[^a-z0-9\-:_]/', '-', $text);
        $text = preg_replace('/^_/', ':_', $text);
        $text = preg_replace('/(?<!:)_/', '-', $text);
        $text = preg_replace('/^\-*/', '', $text);
        $text = preg_replace('/\-*$/', '', $text);
        $text = preg_replace('/[\-]{2,}/', '-', $text);
        $text = preg_replace('/[:]{2,}/', ':', $text);

        $text = str_replace(':-', ':', $text);
        $text = str_replace('-:', ':', $text);
        $text = str_replace('_-', '_', $text);
        $text = str_replace('-_', '_', $text);

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
