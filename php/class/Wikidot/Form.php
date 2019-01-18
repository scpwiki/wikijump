<?php

class Wikidot_Form {
	public $fields = array();
	public $presets = array();

    // keep this up to date with Text_Wiki_Parse_Form!
    public static $FORM_REGEXP = '/\[\[form\]\]\s*\n(.*)\n---\s*\n(.*)\n\[\[\/form\]\]/is';
	
	public static function fromYaml($yamlString, $dataYamlString = null) {
		$form = new self();
		$yaml = Wikidot_Yaml::load($yamlString, true); // forgiving mode ;)
		
		if (is_array($yaml['fields'])) {
			foreach ($yaml['fields'] as $name => $f) {
				unset($first_value);
				if (isset($f['options']) && is_array($f['options'])) {
					$f['type'] = 'select';
					$invalid_keys = array();
					foreach ($f['options'] as $key => $option) {
						if (! is_string($key) && ! is_numeric($key)) {
							$invalid_keys[] = $key;
						} else {
							if (is_string($option) || is_numeric($option)) {
								$option = array('value' => $option);
							}
							if (! is_string($option['value']) && ! is_numeric($option['value'])) {
								$invalid_keys[] = $key;
							} else {
								if (!isset($option['label']) || (! is_string($option['label'] && ! is_numeric($option['label'])))) {
									$option['label'] = $key;
								}
								if (! isset($first_value)) {
									$first_value = $option['value'];
								}
							}
						}
					}
					foreach ($invalid_keys as $invalid_key) {
						unset($f['options'][$invalid_key]);
					}
				} else {
					unset($f['options']);
				}
				if (! isset($f['type']) || ! is_string($f['type']) || empty($f['type'])) {
					$f['type'] == 'text';
					if (isset($f['options'])) {
						$f['type'] == 'select';
					}
				}
				if ($f['type'] == 'select' && (! isset($f['options']) || ! count($f['options']))) {
					$f['type'] = 'text';
				}
				if (isset($f['default']) && $f['type'] == 'select') {
					$def_val = $f['default'];
					if (isset($f['options'][$def_val])) {
						$f['default'] = $f['options'][$def_val]['value'];
					} elseif (! in_array($def_val, $f['options'])) {
						unset($f['default']);
					}
				}
				if (! isset($f['default']) || (! is_string($f['default']) && ! is_numeric($f['default']))) {
					unset($f['default']);
				}
				if (! isset($f['default'])) {
					if (isset($first_value)) {
						$f['default'] = $first_value;
					} else {
						$f['default'] = '';
					}
				}
                if (is_string($f['category']) || is_numeric($f['category'])) {
                    $f['category'] = WDStringUtils::toUnixName($f['category']);
                } else {
                    $f['category'] = '';
                }
                $name = WDStringUtils::toUnixName($name);
                $f['name'] = $name;
				$form->fields[$name] = $f;
			}
			
		}

        $form->setDataFromYaml($dataYamlString);
       
		if (is_array($yaml['presets'])) {
			$form->presets = $yaml['presets'];
		}
		return $form;
	}

    public function setDataFromYaml($dataYamlString) {

        if ($dataYamlString) {
            $data = Wikidot_Yaml::load($dataYamlString, true); // forgiving mode again ;)
        } else {
            $data = array();
        }

        foreach ($this->fields as $name => $field) {
            if (isset($data[$name])) {
                $this->fields[$name]['value'] = $data[$name];
            } else {
                $this->fields[$name]['value'] = null;
            }
        }
 
    }

    public static function fromSource($source) {
        $m = array();
        preg_match(self::$FORM_REGEXP, $source, $m);
        if (count($m)) {
            return self::fromYaml($m[1]);
        } else {
            return null;
        }
    }
	
	public function computeValues($values) {
		$ret = array();
		foreach ($this->fields as $name => $field) {
			if (isset($values[$name]) && (is_string($name) || is_numeric($name))) {
				$value = $values[$name];
				$type = $field['type'];
				
				if ($type == 'select') { 
					if (isset($field['options'][$value])) {
						$ret[$name] = $field['options'][$value]['value'];
					}
				} elseif ($type == 'text') {
					$ret[$name] = $value;
				}
			}
			if (! isset($ret[$name])) {
				$ret[$name] = $field['default'];
			}
		}
		return $ret;
	}
}
