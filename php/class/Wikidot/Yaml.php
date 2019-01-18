<?php

require_once(WIKIDOT_ROOT . "/lib/sfyaml/lib/sfYamlParser.php");
require_once(WIKIDOT_ROOT . "/lib/sfyaml/lib/sfYaml.php");
require_once(WIKIDOT_ROOT . "/lib/spyc/spyc.php");

class Wikidot_Yaml {
	public static function load($string, $forgiving = false) {
        if (substr($string, 0, 3) != '---') {
            $string = "---\n$string";
        }
        try {
            // if syck is available use it
            if (extension_loaded('syck')) {
                return syck_load($string);
            }
            // if not, use the symfony YAML parser
            $yaml = new sfYamlParser();
            return $yaml->parse($string);
        } catch (Exception $e) {
            if ($forgiving) {
                // if YAML document is not correct,
                // but we're forgiving, use the Spyc parser
                return Spyc::YAMLLoadString($string);
            }
            throw new Wikidot_Yaml_Exception("Can't parse the YAML string." . $e->getMessage());
        }
	}
    public static function dump($object) {
        // using the slow (but very compatible) symfony YAML dumper
        $ret = sfYaml::dump($object, 999);
        if (substr($string, 0, 3) == '---') {
            return substr($ret, 4);
        }
        return $ret;
    }
}
