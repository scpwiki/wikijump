<?php
declare(strict_types = 1);

namespace Wikidot\Wikitext\FTML;

use \FFI;

final class FtmlRaw
{
    const HEADER = '/usr/local/include/ftml.h';

    // Singleton management
    private static FFI $ffi;

    // Constant values
    public static int $META_NAME;
    public static int $META_HTTP_EQUIV;
    public static int $META_PROPERTY;

    public static function _init() {
        self::$ffi = FFI::load(self::HEADER);

        // Copy constants
        self::$META_NAME = self::$ffi->META_NAME;
        self::$META_HTTP_EQUIV = self::$ffi->META_HTTP_EQUIV;
        self::$META_PROPERTY = self::$ffi->META_PROPERTY;
    }

    // ftml export methods
    public static function renderHtml(string $wikitext, FtmlPageInfo $page_info): FtmlHtmlOutput {
        $output = self::make('struct ftml_html_output');
        self::$ffi->ftml_render_html(FFI::addr($output), $wikitext, $page_info->pointer());
        return new FtmlHtmlOutput($output);
    }

    public static function renderText(string $wikitext, FtmlPageInfo $page_info): FtmlTextOutput {
        $output = self::make('struct ftml_text_output');
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
    public static function make(string $ctype): FFI\CData {
        return self::$ffi->new($ctype, true, false);
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
function listToPointer(array $list): array {
    // Allocate heap array
    $length = count($list);
    $pointer = FtmlRaw::getInstance()->make("char *[$length]");

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
