<?php
declare(strict_types = 1);

namespace Wikijump\Services\Wikitext;

/**
 * Class FtmlInterface, implements a compatible interface for working with FTML.
 * @package Wikijump\Services\Wikitext
 */
class FtmlBackend implements WikitextBackend
{
    private ?PageInfo $pageInfo;

    public function __construct(string $mode, ?PageInfo $pageInfo) {
        // TODO mode
        $this->pageInfo = $pageInfo;
    }

    public function version(): string {
        return FtmlFfi::version();
    }
}
