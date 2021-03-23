<?php

namespace Wikidot\Utils;

class MailFormUtils
{

    public static function parseFormat($format)
    {
        preg_match_all(
            "/
            ^#\s+                # Start of line, hash, at least one whitespace
            ([a-z0-9_\-]+)       # At least alphanumeric_- char
            \s*:?                # Optional whitespace and colon
            ((?:
                \n(?:\s+\*.*)    # Newline, whitespace, asterisk, then anything
            )+)
            /mix",
            $format,
            $matches,
            PREG_SET_ORDER
        );

        $fields = array();
        foreach ($matches as $f) {
            $field = array();
            $field['name'] = $f[1];

            $parameters = $f[2];

            // ok, should the parameters be parsed? at least some. or all.
            preg_match_all("/
                ^\s\*\s*
                ([a-z0-9\-_]+)
                \s*:\s*(.*)
                $
                /mix", $parameters, $m2, PREG_SET_ORDER);
            foreach ($m2 as $parameter) {
                $field[$parameter[1]] = $parameter[2];
            }

            // if "select", look for options
            if ($field['type'] == "select") {
                preg_match_all("/^ \*\s+options\s*:?((?:\n(?:  \*.*))+)/mi", $parameters, $m3, PREG_SET_ORDER);
                $optionsf = $m3[1];

                preg_match_all("/^  \*\s*([a-z0-9\-_]+)\s*:\s*(.*)$/mi", $parameters, $m4, PREG_SET_ORDER);
                $field['options'] = array();
                foreach ($m4 as $option) {
                    $field['options'][$option[1]] = $option[2];
                }

                if (!in_array($field['default'], array_keys($field['options']))) {
                    unset($field['default']);
                }
            }

            // check if there are any rulezz^H^Hs
            preg_match_all("/^ \*\s+rules\s*:?((?:\n(?:  \*.*))+)/mi", $parameters, $m5, PREG_SET_ORDER);
            if (count($m5)>0) {
                preg_match_all("/^  \*\s*([a-z0-9]+)\s*:\s*(.*)$/mi", $parameters, $m5, PREG_SET_ORDER);
                $field['rules'] = array();
                foreach ($m5 as $rule) {
                    $field['rules'][$rule[1]] = $rule[2];
                }
            }
            $fields[] = $field;
        }
        return $fields;
    }
}
