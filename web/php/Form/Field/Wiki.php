<?php


namespace Wikidot\Form\Field;

use Wikijump\Services\Wikitext\ParseRenderMode;

use function Wikijump\Services\Wikitext\getWikitextBackend;

class Wiki extends Base
{
    public function renderView()
    {
        $source = $this->field['value'];
        $wt = getWikitextBackend(ParseRenderMode::DIRECT_MESSAGE, null);
        return $wt->renderHtml($source)->html;
    }

    public function renderEdit()
    {
        return '<textarea name="field_' . $this->field['name'] . '">' . $this->hvalue() . '</textarea>';
    }
}
