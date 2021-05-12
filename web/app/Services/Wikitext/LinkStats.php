<?php

namespace Wikijump\Services\Wikitext;

use Text_Wiki;

/**
 * Class LinkStats, contains information about links and includes found while parsing and rendering.
 * @package Wikijump\Services\Wikitext
 */
class LinkStats
{
    public int $internalLinksPresent;
    public int $internalLinksAbsent;
    public int $inclusionsPresent;
    public int $inclusionsAbsent;
    public int $externalLinks;

    public function __construct(
        int $internalLinksPresent,
        int $internalLinksAbsent,
        int $inclusionsPresent,
        int $inclusionsAbsent,
        int $externalLinks
    )
    {
        $this->internalLinksPresent = $internalLinksPresent;
        $this->internalLinksAbsent = $internalLinksAbsent;
        $this->inclusionsPresent = $inclusionsPresent;
        $this->inclusionsAbsent = $inclusionsAbsent;
        $this->externalLinks = $externalLinks;
    }

    public static function fromWikiObject(Text_Wiki $wiki): LinkStats
    {
        $internalLinksPresent = $wiki->vars['internalLinksExist'];
        $internalLinksAbsent = $wiki->vars['internalLinksNotExist'];
        $inclusionsPresent = $wiki->vars['inclusions'];
        $inclusionsAbsent = $wiki->vars['inclusionsNotExist'];
        $externalLinks = $wiki->vars['externalLinks'];

        return new LinkStats(
            $internalLinksPresent,
            $internalLinksAbsent,
            $inclusionsPresent,
            $inclusionsAbsent,
            $externalLinks,
        );
    }
}
