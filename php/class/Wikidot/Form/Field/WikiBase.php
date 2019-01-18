<?php

class Wikidot_Form_Field_WikiBase extends Wikidot_Form_Field_Wiki {
    public $rule = "::";
    public function renderView() {
        if (preg_match($this->rule, $this->field['value'])) {
            return parent::renderView();
        } else {
            return '';
        }
    }
}
