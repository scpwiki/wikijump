<?php
declare(strict_types = 1);

namespace Wikijump\Services\Wikitext;

class NullBackend implements WikitextBackend
{
    public function version(): string {
        return 'null 0.0.0';
    }
}
