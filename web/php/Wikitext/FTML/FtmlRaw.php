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

    public static FFI\CType $C_CHAR;
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

        self::$C_CHAR = self::type('char');
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
        return self::$ffi->ftml_version();
    }

    // FFI utilities

    /**
     * Allocate a new instance of the given C object.
     *
     * The object is automatically deallocated once the
     * PHP garbage collector finds no existing references to it.
     *
     * @param FFI\CType $ctype
     * @return FFI\CData
     */
    public static function make(FFI\CType $ctype): FFI\CData {
        return self::$ffi->new($ctype, true, false);
    }

    /**
     * Gets the PHP FFI C type described by the string.
     * See also the static type constants on the class.
     *
     * @param string $type
     * @return FFI\CType
     */
    public static function type(string $type): FFI\CType {
        return self::$ffi->type($type);
    }

    /**
     * Gets the PHP FFI C array type, for the given FFI type and the length.
     *
     * For instance, to produce a 'char[24]', call arrayType(FtmlRaw::C_CHAR, [24]).
     * To produce deep arrays, such as 'int[8][8]', then $dimensions is [8, 8].
     *
     * @param FFI\CType $ctype
     * @param array $dimensions
     * @return FFI\CType
     */
    public static function arrayType(FFI\CType $ctype, array $dimensions): FFI\CType {
        return self::$ffi->arrayType($ctype, $dimensions);
    }

    /**
     * Clones a PHP string into a newly-allocated C string.
     *
     * Not to be confused with FFI::string, which converts
     * a C string into a PHP string.
     *
     * @param ?string $value The string to be cloned, or null
     * @return FFI\CData The C-string created (char *)
     */
    public static function string(?string $value): ?FFI\CData {
        // Check for null
        if (is_null($value)) {
            return null;
        }

        // Allocate C buffer
        $length = strlen($value); // gets bytes, not chars
        $type = FtmlRaw::arrayType(FtmlRaw::C_CHAR, $length + 1);
        $buffer = FtmlRaw::make($type);

        // Copy string data, add null byte
        FFI::memcpy($buffer, $value, $length);
        $buffer[$length] = '\0';
        return $buffer;
    }

    /**
     * Converts a list in the form of a PHP array into a pointer
     * suitable for passing into C FFIs. Applies a transformation
     * to each PHP item to produce the C item.
     *
     * @returns array with keys "pointer" and "length"
     */
    public static function listToPointer(FFI\CType $type, array $list, callable $convertFn): array {
        // Allocate heap array
        $length = count($list);
        $pointerType = FtmlRaw::arrayType($type, [$length]);
        $pointer = FtmlRaw::make($pointerType);

        // Copy string elements
        foreach ($list as $index => $item) {
            $pointer[$index] = $convertFn($item);
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
    public static function pointerToList(FFI\CData $pointer, int $length, callable $convertFn): array {
        $list = array();

        for ($i = 0; $i < $length; $i++) {
            $item = $convertFn($pointer[$i]);
            array_push($list, $item);
        }

        return $list;
    }
}

// Initialize FtmlRaw
FtmlRaw::_init();
