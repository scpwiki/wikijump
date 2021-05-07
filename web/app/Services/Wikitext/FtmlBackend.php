<?php
declare(strict_types = 1);

namespace Wikijump\Services\Wikitext;

/**
 * Class FtmlInterface, implements a compatible interface for working with FTML.
 * @package Wikijump\Services\Wikitext
 */
class FtmlBackend implements WikitextBackend
{
    public function version(): string {
        return FtmlFfi::version();
    }
}
