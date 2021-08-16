<?php
declare(strict_types=1);

namespace Wikijump\Services\Wikitext\FFI;

use \FFI;
use \Wikidot\DB\PagePeer;
use \Wikijump\Services\Wikitext\Backlinks;
use \Wikijump\Services\Wikitext\PageRef;
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
    public static function makeHtmlMetaArray(FFI\CData &$pointer, int $length): array
    {
        return FtmlFfi::pointerToList(
            $pointer,
            $length,
            fn(FFI\CData &$data) => self::makeHtmlMeta($data),
        );
    }

    public static function makeHtmlMeta(FFI\CData &$data): HtmlMeta
    {
        $tagType = self::getTagType($data->tag_type);
        $name = FFI::string($data->name);
        $value = FFI::string($data->value);
        return new HtmlMeta($tagType, $name, $value);
    }

    private static function getTagType(int $tag): string
    {
        switch ($tag) {
            case FtmlFfi::$META_NAME:
                return HtmlMetaType::NAME;
            case FtmlFfi::$META_HTTP_EQUIV:
                return HtmlMetaType::HTTP_EQUIV;
            case FtmlFfi::$META_PROPERTY:
                return HtmlMetaType::PROPERTY;
            default:
                throw new Error("Invalid HTML meta tag type C enum value: $tag");
        }
    }

    // HtmlOutput
    public static function makeHtmlOutput(string $siteId, FFI\CData &$data): HtmlOutput
    {
        $body = FFI::string($data->body);
        $styles = self::makeStylesArray($data->styles_list, $data->styles_len);
        $meta = self::makeHtmlMetaArray($data->meta_list, $data->meta_len);
        $warnings = self::makeParseWarningArray($data->warning_list, $data->warning_len);
        $backlinks = self::makeBacklinks($siteId, $data->backlinks);

        // Free original C data
        FtmlFfi::freeHtmlOutput($data);
        FFI::free($data);

        // Return object
        return new HtmlOutput($body, $styles, $meta, $warnings, $backlinks);
    }

    private static function makeStylesArray(FFI\CData &$pointer, int $length): array {
        return FtmlFfi::pointerToList(
            $pointer,
            $length,
            fn(FFI\CData $data) => FFI::string($data),
        );
    }

    // TextOutput
    public static function makeTextOutput(FFI\CData &$data): TextOutput
    {
        $text = FFI::string($data->text);
        $warnings = self::makeParseWarningArray($data->warning_list, $data->warning_len);

        // Free original C data
        FtmlFfi::freeTextOutput($data);
        FFI::free($data);

        // Return object
        return new TextOutput($text, $warnings);
    }

    // ParseWarning
    public static function makeParseWarningArray(FFI\CData &$pointer, int $length): array
    {
        return FtmlFfi::pointerToList(
            $pointer,
            $length,
            fn(FFI\CData $data) => self::makeParseWarning($data),
        );
    }

    public static function makeParseWarning(FFI\CData &$data): ParseWarning
    {
        $token = FFI::string($data->token);
        $rule = FFI::string($data->rule);
        $spanStart = $data->span_start;
        $spanEnd = $data->span_end;
        $kind = FFI::string($data->kind);
        return new ParseWarning($token, $rule, $spanStart, $spanEnd, $kind);
    }

    // Backlinks
    public static function makeBacklinks(string $siteId, FFI\CData &$data): Backlinks
    {
        $inclusions = self::splitLinks(
            $data->included_pages_list,
            $data->included_pages_len,
            fn(FFI\CData $data) => self::makePageRef($data),
            fn(PageRef $pageRef) => self::getPageId($siteId, $pageRef),
        );
        $inclusionsPresent = $inclusions['present'];
        $inclusionsAbsent = $inclusions['absent'];

        $internalLinks = self::splitLinks(
            $data->internal_links_list,
            $data->internal_links_len,
            fn(FFI\CData $data) => self::makePageRef($data),
            fn(PageRef $pageRef) => self::getPageId($siteId, $pageRef),
        );
        $internalLinksPresent = $internalLinks['present'];
        $internalLinksAbsent = $internalLinks['absent'];

        $externalLinks = FtmlFfi::pointerToList(
            $data->external_links_list,
            $data->external_links_len,
            fn(FFI\CData $data) => FFI::string($data),
        );

        return new Backlinks(
            $inclusionsPresent,
            $inclusionsAbsent,
            $internalLinksPresent,
            $internalLinksAbsent,
            $externalLinks,
        );
    }

    private static function makePageRef(FFI\CData &$data): PageRef
    {
        $site = FtmlFfi::nullableString($data->site);
        $page = FFI::string($data->page);

        return new PageRef($site, $page);
    }

    private static function splitLinks(
        FFI\CData &$pointer,
        int $length,
        callable $convertFn,
        callable $checkItemFn
    ): array {
        $present = [];
        $absent = [];

        // Convert items, placing in the appropriate list
        for ($i = 0; $i < $length; $i++) {
            $originalItem = $convertFn($pointer[$i]);
            $foundItem = $checkItemFn($originalItem);

            if ($foundItem === null) {
                array_push($absent, $originalItem);
            } else {
                array_push($present, $foundItem);
            }
        }

        return [
            "present" => $present,
            "absent" => $absent,
        ];
    }

    private static function getPageId(string $siteId, PageRef &$pageRef): ?string
    {
        if ($pageRef->site !== null) {
            $siteId = FtmlFfi::getSiteId($pageRef->site);
            if ($siteId === null) {
                // Site not found, the page obviously doesn't exist
                return null;
            }
        }

        // Find the page based on the contextual site ID
        $page = PagePeer::instance()->selectByName($siteId, $pageRef->page);
        return $page ? $page->getPageId() : null;
    }
}
