<?php
declare(strict_types=1);

namespace Wikijump\Services\Wikitext;

use Ds\Set;
use Wikidot\DB\Page;

/**
 * Class PageInfo, representing information associated with a page for parsing and rendering.
 * @package Wikijump\Services\Wikitext
 */
class PageInfo
{
    public string $page;
    public ?string $category;
    public string $site;
    public string $title;
    public ?string $alt_title;
    public Set $tags;
    public string $language;

    public function __construct(
        string $page,
        ?string $category,
        string $site,
        string $title,
        ?string $alt_title,
        Set $tags,
        string $language
    ) {
        $this->page = $page;
        $this->category = $category;
        $this->site = $site;
        $this->title = $title;
        $this->alt_title = $alt_title;
        $this->tags = $tags;
        $this->language = $language;
    }

    public static function fromPageObject(Page $page): PageInfo
    {
        $page_slug = $page->getUnixName();
        $category_slug = $page->getCategoryName();
        $site_slug = $page->getSite()->getUnixName();
        $title = $page->getTitle();
        $alt_title = null;
        $tags = $page->getTags();
        $language = 'default';

        return new PageInfo(
            $page_slug,
            $category_slug,
            $site_slug,
            $title,
            $alt_title,
            $tags,
            $language,
        );
    }

    public function getCategory(): string
    {
        return $this->category ?? '_default';
    }

    public function getPageSlug(): string
    {
        return $this->category === null
            ? $this->page
            : $this->category . ':' . $this->page;
    }
}
