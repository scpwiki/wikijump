<?php

class Wikidot_Form_Field_Page extends Wikidot_Form_Field_Base {
    public function renderView() {
        return "<a href=\"/{$this->data}\">{$this->data}</a>";
    }
}
