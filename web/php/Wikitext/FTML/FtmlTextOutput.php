<?php
declare(strict_types = 1);

namespace Wikidot\Wikitext\FTML;

use \FFI;

class FtmlTextOutput
{
    public string $text;
    public array $warnings;

    public function __construct(FFI\CData $c_data) {
        $this->text = FFI::string($c_data->text);
        $this->warnings = FtmlWarning::fromArray($c_data->warning_list, $c_data->warning_len);

        // Free original C data
        FtmlRaw::freeTextOutput($c_data);
    }
}
