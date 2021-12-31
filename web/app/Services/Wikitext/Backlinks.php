<?php
declare(strict_types=1);

namespace Wikijump\Services\Wikitext;

use Text_Wiki;

/**
 * Class Backlinks, contains information about links found while parsing and rendering.
 * @package Wikijump\Services\Wikitext
 */
class Backlinks
{
    public array $inclusions;
    public array $internal_links;
    public array $external_links;

    public function __construct(
        array $inclusions,
        array $internal_links,
        array $external_links
    ) {
        $this->inclusions = $inclusions;
        $this->internal_links = $internal_links;
        $this->external_links = $external_links;
    }

    public static function fromWikiObject(Text_Wiki $wiki): Backlinks
    {
        $inclusions = $wiki->vars['inclusions'] ?? [];
        $internal_links = $wiki->vars['internalLinks'] ?? [];
        $external_links = $wiki->vars['externalLinks'] ?? [];

        return new Backlinks(
            $inclusions,
            $internal_links,
            $external_links,
        );
    }
}
