<?php
declare(strict_types = 1);

namespace Wikidot\Wikitext\FTML;

use \FFI;

class FtmlRaw
{
    const HEADER = '/usr/local/include/ftml.h';
    const LIBRARY = '/usr/local/lib/libftml.so';

    // Singleton management and creation
    private static ?FtmlRaw $instance = null;

    public static function getInstance(): FtmlRaw {
        if (self::$instance == null) {
            self::$instance = new FtmlRaw();
        }

        return self::$instance;
    }

    private FFI $ffi;

    private function __construct() {
        $this->ffi = FFI::cdef(FtmlRaw::HEADER, FtmlRaw::LIBRARY);
    }

    // ftml export methods
    public function renderHtml(string $wikitext, FtmlPageInfo $page_info): FtmlHtmlOutput {
        $output = $this->make('struct ftml_html_output');
        $this->ffi->ftml_render_html($output, $wikitext, $page_info);
        return new FtmlHtmlOutput($output);
    }

    public function renderText(string $wikitext, FtmlPageInfo $page_info): FtmlTextOutput {
        $output = $this->make('struct ftml_text_output');
        $this->ffi->ftml_render_text($output, $wikitext, $page_info);
        return new FtmlTextOutput($output);
    }

    public function freeHtmlOutput(FFI\CData $c_data) {
        $this->ffi->ftml_destroy_html_output($c_data);
    }

    public function freeTextOutput(FFI\CData $c_data) {
        $this->ffi->ftml_destroy_text_output($c_data);
    }

    public function version(): string {
        return FFI::string($this->ffi.ftml_version());
    }

    // FFI utilities
    public function make(string $ctype): FFI\CData {
        return $this->ffi->new($ctype, true, false);
    }

    // Constant values
    public static function metaName(): FFI\CData {
        return self::getInstance()->ffi->META_NAME;
    }

    public static function metaHttpEquiv(): FFI\CData {
        return self::getInstance()->ffi->META_HTTP_EQUIV;
    }

    public static function metaProperty(): FFI\CData {
        return self::getInstance()->ffi->META_PROPERTY;
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
function pointerToList(FFI\CData $pointer, int $length, callable $convert_fn): array {
    $list = array();

    for ($i = 0; $i < $length; $i++) {
        $item = $convert_fn($pointer[$i]);
        array_push($list, $item);
    }

    return $list;
}
