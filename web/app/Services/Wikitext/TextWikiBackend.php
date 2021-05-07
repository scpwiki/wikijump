<?php
declare(strict_types = 1);

namespace Wikijump\Services\Wikitext;

use \Wikidot\Utils\WikiTransformation;

class TextWikiBackend implements WikitextBackend
{
    public function version(): string {
        return 'Text_Wiki 0.0.1';
    }
}
