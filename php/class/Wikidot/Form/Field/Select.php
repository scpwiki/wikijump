<?php

class Wikidot_Form_Field_Select extends Wikidot_Form_Field_Base {
    public function renderView() {
        return htmlspecialchars($this->field['options'][$this->field['value']]);
    }
    public function renderEdit() {
        $f = $this->field;
        $output = "<select name=\"field_$f[name]\">";
        foreach ($f['options'] as $name => $option) {
            if ($name == $f['value']) {
                $selected = ' selected="selected"';
            } else {
                $selected = '';
            }
            $output .= '<option value="' . htmlspecialchars($name) . '"' . $selected . '>' . htmlspecialchars($option) . '</option>';
        }
        return "$output</select>";
    }
}
