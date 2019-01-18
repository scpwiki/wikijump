<?php

class Wikidot_Form_Renderer extends Wikidot_Form {
    public function __construct($form) {
        $fields = $form->fields;
        $this->presets = $form->presets;
        $this->data = $form->data;
    
        foreach ($fields as $name => $field) {
            $this->fields[$name] = $field;
            $this->fields[$name]['editor'] = Wikidot_Form_Field::field($field);
        }
    }
}
