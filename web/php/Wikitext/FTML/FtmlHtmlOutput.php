<?php

namespace Wikidot\Wikitext\FTML;

class FtmlHtmlOutput
{
    private FFI\CData $c_data;

    public function __construct(FFI\CData $c_data) {
        // TODO convert to PHP
        $this->c_data = $c_data;
    }

    function __destruct() {
        parent::__destruct();
        FtmlRaw::getInstance()->freeHtmlOutput($this->c_data);
    }
}