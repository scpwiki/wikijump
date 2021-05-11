<?php
declare(strict_types = 1);

namespace Wikijump\Services\Wikitext;

use Wikidot\Utils\GlobalProperties;

interface WikitextBackend
{
    public function version(): string;
}

/**
 * Gets the WikitextBackend interface to allow for parsing, rendering, and related
 * wikitext transformation.
 *
 * @throws GlobalPropertiesException if the feature flag value is invalid
 */
function getWikitext(): WikitextBackend {
    switch (GlobalProperties::$FEATURE_WIKITEXT_BACKEND) {
        case 'text_wiki':
            return new TextWikiBackend();
        case 'ftml':
            return new FtmlBackend();
        case 'null':
            return new NullBackend();
        default:
            throw new Exception('Wikitext backend feature flag invalid: ' . GlobalProperties::$FEATURE_WIKITEXT_BACKEND);
    }
}
