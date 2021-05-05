<?php
declare(strict_types = 1);

namespace Wikidot\Wikitext\FTML;

use \FFI;

/**
 * Class FtmlTextOutput, representing a returned 'struct ftml_text_output' object.
 * @package Wikidot\Wikitext\FTML
 */
class FtmlTextOutput
{
    public string $text;
    public array $warnings;

    public function __construct(FFI\CData $c_data) {
        $this->text = FFI::string($c_data->text);
        $this->warnings = FtmlWarning::fromArray($c_data->warning_list, $c_data->warning_len);

        // Free original C data
        FtmlFfi::freeTextOutput($c_data);
        FFI::free($c_data);
    }
}
