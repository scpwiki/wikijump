<?php
declare(strict_types = 1);

namespace Wikijump\Services\Wikitext;

/**
 * Class HtmlMeta, represents one &lt;meta&gt;, to be included in a rendered HTML document.
 * @package Wikijump\Services\Wikitext
 */
class HtmlMeta
{
    /**
     * @var string The kind of HTML meta tag.
     *
     * Can be one of:
     * - 'name'
     * - 'http-equiv'
     * - 'property'
     */
    public string $tagType;

    /**
     * @var string The HTML meta tag's key.
     */
    public string $name;

    /**
     * @var string The HTML meta tag's value.
     */
    public string $value;
}
