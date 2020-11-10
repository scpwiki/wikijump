<?php


namespace Wikijump\Form\Field;

class Text extends Base
{
    public function renderView()
    {
        return $this->hvalue();
    }
    public function renderEdit()
    {
        return '<input class="text" type="text" value="' . $this->hvalue() . '" name="field_' . $this->field['name'] . '"/>';
    }
}
