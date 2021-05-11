<?php
declare(strict_types = 1);

namespace Wikijump\Services\Wikitext\FFI;

use \FFI;

/**
 * Class ParseWarning, representing a returned 'struct ftml_warning' object.
 * @package Wikijump\Services\Wikitext\FFI
 */
class ParseWarning
{
    public string $token;
    public string $rule;
    public int $spanStart;
    public int $spanEnd;
    public string $kind;

    public function __construct(FFI\CData $c_data) {
        $this->token = FFI::string($c_data->token);
        $this->rule = FFI::string($c_data->rule);
        $this->spanStart = $c_data->span_start;
        $this->spanEnd = $c_data->span_end;
        $this->kind = FFI::string($c_data->kind);
    }

    public static function fromArray(FFI\CData $pointer, int $length): array {
        return FtmlFfi::pointerToList(
            $pointer,
            $length,
            fn(FFI\CData $c_data) => new ParseWarning($c_data),
        );
    }
}
