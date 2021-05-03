<?php
declare(strict_types = 1);

namespace Wikidot\Wikitext;

use Wikidot\Wikitext\FtmlPageInfo;

class FtmlRaw
{
    const HEADER = '/usr/local/include/ftml.h';
    const LIBRARY = '/usr/local/lib/libftml.so';

    // Singleton management and creation
    private static ?FtmlRaw $instance;

    public static function getInstance(): FtmlRaw {
        if (FtmlRaw::$instance == null) {
            FtmlRaw::$instance = new FtmlRaw();
        }

        return FtmlRaw::$instance;
    }

    private FFI $ffi;

    private function __construct() {
        $this->ffi = FFI::cdef(FtmlRaw::HEADER, FtmlRaw::LIBRARY);
    }

    // ftml export methods
    public function renderHtml(string $wikitext, FtmlPageInfo $page_info) {
        // TODO
    }

    public function renderText(string $wikitext, FtmlPageInfo $page_info) {
        // TODO
    }

    public function freeHtmlOutput(FFI\CData $c_data) {
        // TODO
    }

    public function freeTextOutput(FFI\CData $c_data) {
        // TODO
    }

    public function version(): string {
        return FFI::string($this->ffi.ftml_version());
    }

    // FFI utilities
    public function make(string $ctype): FFI\CData {
        return $this->ffi->new($ctype, true, false);
    }
}
