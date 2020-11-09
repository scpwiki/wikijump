<?php


namespace Wikijump\Form\Field;

use \WikiTransformation;

class Wiki extends Base
{
    public function renderView()
    {
        $wt = new WikiTransformation();
        $wt->setMode('pm');
        return $wt->processSource($this->field['value']);
    }
    public function renderEdit()
    {
        return '<textarea name="field_' . $this->field['name'] . '">' . $this->hvalue() . '</textarea>';
    }
}
