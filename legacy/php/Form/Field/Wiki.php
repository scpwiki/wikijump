<?php


namespace Wikidot\Form\Field;

use Wikijump\Services\Deepwell\DeepwellService;
use Wikijump\Services\Wikitext\ParseRenderMode;

class Wiki extends Base
{
    public function renderView()
    {
        $source = $this->field['value'];
        return DeepwellService::getInstance()->renderHtml(ParseRenderMode::DIRECT_MESSAGE, $source, null);
    }

    public function renderEdit()
    {
        return '<textarea name="field_' . $this->field['name'] . '">' . $this->hvalue() . '</textarea>';
    }
}
