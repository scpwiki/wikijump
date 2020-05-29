<?php


namespace Wikidot\Form;

use Wikidot\Form;
use Wikidot\Form\Field;



class Renderer extends Form {
    public function __construct($form) {
        $fields = $form->fields;
        $this->presets = $form->presets;
        $this->data = $form->data;
    
        foreach ($fields as $name => $field) {
            $this->fields[$name] = $field;
            $this->fields[$name]['editor'] = Field::field($field);
        }
    }
}
