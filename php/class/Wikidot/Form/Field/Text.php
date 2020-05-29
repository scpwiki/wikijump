<?php


namespace Wikidot\Form\Field;

use Wikidot\Form\Field\Base;



class Text extends Base {
    public function renderView() {
        return $this->hvalue();
    }
    public function renderEdit() {
        return '<input class="text" type="text" value="' . $this->hvalue() . '" name="field_' . $this->field['name'] . '"/>';
    }
}
