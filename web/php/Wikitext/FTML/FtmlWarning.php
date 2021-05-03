<?php

namespace Wikidot\Wikitext\FTML;

class FtmlWarning
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
        return pointerToList(
            $pointer,
            $length,
            fn(FFI\CData $c_data) => new FtmlWarning($c_data),
        );
    }
}
