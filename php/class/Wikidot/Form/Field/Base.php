<?php

class Wikidot_Form_Field_Base {
    protected $field;

    public function __construct($field) {
        $this->field = $field;
    }

    public function renderView() {
        return '';
    }

    public function renderEdit() {
        return '';
    }

    protected function hvalue() {
        return htmlspecialchars($this->field['value']);
    }
}
