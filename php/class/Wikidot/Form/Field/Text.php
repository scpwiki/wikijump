<?php

class Wikidot_Form_Field_Text extends Wikidot_Form_Field_Base {
    public function renderView() {
        return $this->hvalue();
    }
    public function renderEdit() {
        return '<input class="text" type="text" value="' . $this->hvalue() . '" name="field_' . $this->field['name'] . '"/>';
    }
}
