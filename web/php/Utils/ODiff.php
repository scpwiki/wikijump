<?php

namespace Wikidot\Utils;

/**
 * A wrapper around xdiff.
 */
class ODiff
{
    private $contextLines = 1;
    private $minimal = true;

    private $errors = null;

    public function setContextLines($val)
    {
        $this->contextLines = $val;
    }

    public function setMinimal($val)
    {
        $this->minimal = $val;
    }

    public function getErrors()
    {
        return $this->errors;
    }

    public function diffString($stringFrom, $stringTo)
    {
        // fix "no new lineat the end" problem.
        if (!preg_match("/\n$/", $stringFrom)) {
            $stringFrom.="\n";
        }
        if (!preg_match("/\n$/", $stringTo)) {
            $stringTo.="\n";
        }
        return xdiff_string_diff($stringFrom, $stringTo);
    }

    public function patchString($string, $patch, $reverse = false)
    {
        if (!preg_match("/\n$/", $string)) {
            $string.="\n";
        }
        if (!preg_match("/\n$/", $patch)) {
            $patch.="\n";
        }
        if ($reverse == false) {
            return xdiff_string_patch($string, $patch);
        } else {
            return xdiff_string_patch($string, $patch, XDIFF_PATCH_REVERSE);
        }
    }
}
