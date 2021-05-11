<?php
declare(strict_types = 1);

namespace Wikijump\Services\Wikitext\FFI;

use \FFI;

/**
 * Class TextOutput, representing a returned 'struct ftml_text_output' object.
 * @package Wikijump\Services\Wikitext\FFI
 */
class TextOutput
{
    public string $text;
    public array $warnings;

    public function __construct(FFI\CData $c_data) {
        $this->text = FFI::string($c_data->text);
        $this->warnings = ParseWarning::fromArray($c_data->warning_list, $c_data->warning_len);

        // Free original C data
        FtmlFfi::freeTextOutput($c_data);
        FFI::free($c_data);
    }
}
