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
     * @var string The kind of HTML meta tag. See HtmlMetaType for details.
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
