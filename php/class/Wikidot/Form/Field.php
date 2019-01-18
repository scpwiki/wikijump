<?php

class Wikidot_Form_Field {
    static public function field($field) {
        $t = $field['type'];
        if ($t == 'text') return new Wikidot_Form_Field_Text($field);
        if ($t == 'wiki') return new Wikidot_Form_Field_Wiki($field);
        if ($t == 'select') return new Wikidot_Form_Field_Select($field);
        if ($t == 'page') return new Wikidot_Form_Field_Page($field);
        if ($t == 'pagepath') return new Wikidot_Form_Field_PagePath($field);
    }
}

