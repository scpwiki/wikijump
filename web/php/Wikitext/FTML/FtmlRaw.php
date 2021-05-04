<?php
declare(strict_types = 1);

namespace Wikidot\Wikitext\FTML;

use \FFI;

/**
 * Class FtmlRaw, for interacting directly with the FTML FFI.
 * You probably want to use FtmlBackend instead.
 *
 * @package Wikidot\Wikitext\FTML
 */
final class FtmlRaw
{
    const HEADER = '/usr/local/include/ftml.h';

    // Singleton management
    private static FFI $ffi;

    // Constant values
    public static int $META_NAME;
    public static int $META_HTTP_EQUIV;
    public static int $META_PROPERTY;

    public static FFI\CType $C_STRING;
    public static FFI\CType $FTML_PAGE_INFO;
    public static FFI\CType $FTML_HTML_OUTPUT;
    public static FFI\CType $FTML_TEXT_OUTPUT;

    public static function _init() {
        self::$ffi = FFI::load(self::HEADER);

        // Create constants
        self::$META_NAME = self::$ffi->META_NAME;
        self::$META_HTTP_EQUIV = self::$ffi->META_HTTP_EQUIV;
        self::$META_PROPERTY = self::$ffi->META_PROPERTY;

        self::$C_STRING = self::type('char *');
        self::$FTML_PAGE_INFO = self::type('struct ftml_page_info');
        self::$FTML_HTML_OUTPUT = self::type('struct ftml_html_output');
        self::$FTML_TEXT_OUTPUT = self::type('struct ftml_text_output');
    }

    // ftml export methods
    public static function renderHtml(string $wikitext, FtmlPageInfo $page_info): FtmlHtmlOutput {
        $output = self::make(self::$FTML_HTML_OUTPUT);
        self::$ffi->ftml_render_html(FFI::addr($output), $wikitext, $page_info->pointer());
        return new FtmlHtmlOutput($output);
    }

    public static function renderText(string $wikitext, FtmlPageInfo $page_info): FtmlTextOutput {
        $output = self::make(self::$FTML_TEXT_OUTPUT);
        self::$ffi->ftml_render_text(FFI::adr($output), $wikitext, $page_info->pointer());
        return new FtmlTextOutput($output);
    }

    public static function freeHtmlOutput(FFI\CData $c_data) {
        self::$ffi->ftml_destroy_html_output(FFI::addr($c_data));
    }

    public static function freeTextOutput(FFI\CData $c_data) {
        self::$ffi->ftml_destroy_text_output(FFI::addr($c_data));
    }

    public static function version(): string {
        return FFI::string(self::$ffi->ftml_version());
    }

    // FFI utilities
    public static function make(FFI\CType $ctype): FFI\CData {
        return self::$ffi->new($ctype, true, false);
    }

    public static function type(string $type): FFI\CType {
        return self::$ffi->type($type);
    }

    public static function arrayType(FFI\CType $ctype, array $dimensions): FFI\CType {
        return self::$ffi->arrayType($ctype, $dimensions);
    }
}

/**
 * Converts a list in the form of a PHP array into a pointer
 * suitable for passing into C FFIs.
 *
 * All of the objects in the array must already be ready for passing.
 *
 * @returns array with keys "pointer" and "length"
 */
function listToPointer(FFI\CType $type, array $list): array {
    // Allocate heap array
    $length = count($list);
    $pointerType = FtmlRaw::arrayType($type, [$length]);
    $pointer = FtmlRaw::make($pointerType);

    // Copy string elements
    foreach ($list as $index => $item) {
        $pointer[$index] = $item;
    }

    return [
        'pointer' => $pointer,
        'length' => $length,
    ];
}

/**
 * Converts a C FFI pointer into a PHP array, applying a transformation
 * to each C item to produce the PHP object.
 *
 * @returns array with the converted objects
 */
function pointerToList(FFI\CData $pointer, int $length, callable $convertFn): array {
    $list = array();

    for ($i = 0; $i < $length; $i++) {
        $item = $convertFn($pointer[$i]);
        array_push($list, $item);
    }

    return $list;
}

// Initialize FtmlRaw
FtmlRaw::_init();
