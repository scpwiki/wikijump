<?php

namespace Wikidot\Utils;

class WDStringUtils
{
    public static function toUnixName($text)
    {
        $text = trim($text);

        // and absolutely purify the string removing all unwanted characters
        $text = strtolower($text);
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
