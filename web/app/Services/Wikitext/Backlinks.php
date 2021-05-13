<?php

namespace Wikijump\Services\Wikitext;

use Text_Wiki;

/**
 * Class Backlinks, contains information about links found while parsing and rendering.
 * @package Wikijump\Services\Wikitext
 */
class Backlinks
{
    public array $internalLinksPresent;
    public array $internalLinksAbsent;
    public array $inclusionsPresent;
    public array $inclusionsAbsent;
    public array $externalLinks;

    public function __construct(
        array $internalLinksPresent,
        array $internalLinksAbsent,
        array $inclusionsPresent,
        array $inclusionsAbsent,
        array $externalLinks
    )
    {
        $this->internalLinksPresent = $internalLinksPresent;
        $this->internalLinksAbsent = $internalLinksAbsent;
        $this->inclusionsPresent = $inclusionsPresent;
        $this->inclusionsAbsent = $inclusionsAbsent;
        $this->externalLinks = $externalLinks;
    }

    public static function fromWikiObject(Text_Wiki $wiki): Backlinks
    {
        $internalLinksPresent = $wiki->vars['internalLinksExist'] ?? [];
        $internalLinksAbsent = $wiki->vars['internalLinksNotExist'] ?? [];
        $inclusionsPresent = $wiki->vars['inclusions'] ?? [];
        $inclusionsAbsent = $wiki->vars['inclusionsNotExist'] ?? [];
        $externalLinks = $wiki->vars['externalLinks'] ?? [];

        return new Backlinks(
            $internalLinksPresent,
            $internalLinksAbsent,
            $inclusionsPresent,
            $inclusionsAbsent,
            $externalLinks,
        );
    }
}
