<?php
declare(strict_types=1);

namespace Wikijump\Services\Wikitext\FFI;

use \FFI;
use \Wikijump\Services\Wikitext\Backlinks;
use \Wikijump\Services\Wikitext\HtmlMeta;
use \Wikijump\Services\Wikitext\HtmlMetaType;
use \Wikijump\Services\Wikitext\HtmlOutput;
use \Wikijump\Services\Wikitext\ParseWarning;

/**
 * Class OutputConversion, converts various FFI objects into their corresponding PHP output objects.
 * @package Wikijump\Services\Wikitext\FFI
 */
final class OutputConversion
{
    private function __construct() {}

    // HtmlMeta
    public static function makeHtmlMetaArray(FFI\CData $pointer, int $length): array
    {
        return FtmlFfi::pointerToList(
            $pointer,
            $length,
            fn(FFI\CData $c_data) => self::makeHtmlMeta($c_data),
        );
    }

    public static function makeHtmlMeta(FFI\CData $c_data): HtmlMeta
    {
        $tagType = self::getTagType($c_data->tag_type);
        $name = FFI::string($c_data->name);
        $value = FFI::string($c_data->value);
        return new HtmlMeta($tagType, $name, $value);
    }

    private static function getTagType(int $c_tag): string
    {
        switch ($c_tag) {
            case FtmlFfi::$META_NAME:
                return HtmlMetaType::NAME;
            case FtmlFfi::$META_HTTP_EQUIV:
                return HtmlMetaType::HTTP_EQUIV;
            case FtmlFfi::$META_PROPERTY:
                return HtmlMetaType::PROPERTY;
            default:
                throw new Error("Invalid HTML meta tag type C enum value: $c_tag");
        }
    }

    // HtmlOutput
    public static function makeHtmlOutput(FFI\CData $c_data): HtmlOutput
    {
        $body = FFI::string($c_data->body);
        $styles = self::makeStylesArray($c_data->styles_list, $c_data->styles_len);
        $meta = self::makeHtmlMetaArray($c_data->meta_list, $c_data->meta_len);
        $warnings = self::makeParseWarningArray($c_data->warning_list, $c_data->warning_len);

        // Free original C data
        FtmlFfi::freeHtmlOutput($c_data);
        FFI::free($c_data);

        // TODO actually get link information
        $backlinks = new Backlinks([], [], [], [], []);

        // Return object
        return new HtmlOutput($body, $styles, $meta, $warnings, $backlinks);
    }

    private static function makeStylesArray(FFI\CData $pointer, int $length): array {
        return FtmlFfi::pointerToList(
            $pointer,
            $length,
            fn(FFI\CData $c_data) => FFI::string($c_data),
        );
    }

    // TextOutput
    public static function makeTextOutput(FFI\CData $c_data): TextOutput
    {
        $text = FFI::string($c_data->text);
        $warnings = self::makeParseWarningArray($c_data->warning_list, $c_data->warning_len);

        // Free original C data
        FtmlFfi::freeTextOutput($c_data);
        FFI::free($c_data);

        // Return object
        return new TextOutput($text, $warnings);
    }

    // ParseWarning
    public static function makeParseWarningArray(FFI\CData $pointer, int $length): array
    {
        return FtmlFfi::pointerToList(
            $pointer,
            $length,
            fn(FFI\CData $c_data) => self::makeParseWarning($c_data),
        );
    }

    public static function makeParseWarning(FFI\CData $c_data): ParseWarning
    {
        $token = FFI::string($c_data->token);
        $rule = FFI::string($c_data->rule);
        $spanStart = $c_data->span_start;
        $spanEnd = $c_data->span_end;
        $kind = FFI::string($c_data->kind);
        return new ParseWarning($token, $rule, $spanStart, $spanEnd, $kind);
    }
}
