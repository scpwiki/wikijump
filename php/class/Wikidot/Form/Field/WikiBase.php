<?php


namespace Wikidot\Form\Field;

use Wikidot\Form\Field\Wiki;



class WikiBase extends Wiki {
    public $rule = "::";
    public function renderView() {
        if (preg_match($this->rule, $this->field['value'])) {
            return parent::renderView();
        } else {
            return '';
        }
    }
}
