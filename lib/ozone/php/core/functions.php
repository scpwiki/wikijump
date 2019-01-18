<?php

/**
 * Wikidot - free wiki collaboration software
 * Copyright (c) 2008, Wikidot Inc.
 * 
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * For more information about licensing visit:
 * http://www.wikidot.org/license
 * 
 * @category Ozone
 * @package Ozone_Util
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

/**
 * File list utility function.
 * ls(dir,pattern) return file list in "dir" folder matching "pattern"
 * ls("path","module.php?") search into "path" folder for module.php3, module.php4, ...
 * ls("images/","*.jpg") search into "images" folder for JPG images
 *
 * @param string $__dir
 * @param string $__pattern
 * @return array
 */
function ls($__dir = "./", $__pattern = "*.*") {
    settype($__dir, "string");
    settype($__pattern, "string");
    
    $__ls = array();
    $__regexp = preg_quote($__pattern, "/");
    $__regexp = preg_replace("/[\\x5C][\x2A]/", ".*", $__regexp);
    $__regexp = preg_replace("/[\\x5C][\x3F]/", ".", $__regexp);
    
    if (is_dir($__dir))
        if (($__dir_h = @opendir($__dir)) !== FALSE) {
            while (($__file = readdir($__dir_h)) !== FALSE)
                if (preg_match(
                        "/^" .
                                 $__regexp .
                                 "$/", 
                                $__file))
                            array_push(
                                    $__ls, 
                                    $__file);
            
            closedir($__dir_h);
            sort($__ls, SORT_STRING);
        }
    
    return $__ls;
}

function lsreg($__dir = "./", $__pattern = ".*") {
    settype($__dir, "string");
    settype($__pattern, "string");
    
    $__ls = array();
    if (is_dir($__dir))
        if (($__dir_h = @opendir($__dir)) !== FALSE) {
            while (($__file = readdir($__dir_h)) !== FALSE)
                if (preg_match(
                        $__pattern, 
                        $__file))
                    array_push(
                            $__ls, 
                            $__file);
            
            closedir($__dir_h);
            sort($__ls, SORT_STRING);
        }
    
    return $__ls;
}

function underscoreToLowerCase($string) {
    $out = '';
    $string = trim($string);
    $nextUpper = false;
    for ($i = 0; $i < strlen($string); $i++) {
        if ($string{$i} != '_') {
            if ($nextUpper) {
                $out .= strtoupper(
                        $string{$i});
                $nextUpper = false;
            } else {
                $out .= strtolower(
                        $string{$i});
            }
        } else {
            $nextUpper = true;
        }
    }
    
    return $out;
}

function lowerCaseToUnderscore($string) {
    $out = '';
    $string = trim($string);
    for ($i = 0; $i < strlen($string); $i++) {
        if (ctype_upper($string{$i}) && $i != 0) {
            $out .= '_';
        }
        $out .= strtoupper($string{$i});
    }
    return $out;
}

function capitalizeFirstLetter($string) {
    $out = trim($string);
    $out{0} = strtoupper($out{0});
    return $out;
}

function findNodeWithAttribute($list, $attrName, $attrValue) {
    foreach ($list as $node) {
        if ($node[$attrName] == $attrValue) {
            return $node;
        }
    }
    return null;
}

/**
 * Creates the directory by recurence. Starting from leftmost directory given (e.g. when
 * supplied /var/www/ozone/sampleapp/tmp) it checks if all parrent directories exists 
 * (e.g. /var, /var/www, ...) and if not - creates them.
 * @param string $dir name of the directory (absolute preferred)
 */
function mkdirfull($dir) {
    //split by '/' and check if all subsequent parrent directories exist. if not - create them all.
    /* remove duplicated '/', e.g. in /path//subdir */
	/* and remove trailing slash too. */
	$dir = preg_replace(';/{2,};', '/', $dir);
	$dir = preg_replace(';/$;', '', $dir);
    $splited = split('/', $dir);
    $n = count($splited);
    $dir0 = '';
    for ($i = 0; $i < $n; $i++) {
        $dir0 .= '/' . $splited[$i];
        // check if exists
        if (!file_exists(
                $dir0)) {
            mkdir($dir0);
        } else {
            if (is_dir($dir0)) {    //TODO: throw some exception - file exists but is NOT a directory	
            }
        }
    }
}

/**
 * Handle proper string escaping for use in SQL statements for different databases.
 * Type of the database is taken from the GlobalProperties class and appropriate escaping
 * function is used on the input argument.
 * @param string $val input (unescaped) string
 * @return string escaped string
 */

function db_escape_string($val) {
    if ($val === null) {
        return null;
    }
    $databaseType = GlobalProperties::$DATABASE_TYPE;
    if ($databaseType == "mysql") {
        return mysql_real_escape_string($val);
    }
    if ($databaseType == "pgsql") {
        return pg_escape_string($val);
    }
    if ($databaseType == "sqlite") {
        return sqlite_escape_string($val);
    }
    return null;

}

/**
 * Useful function to extract text value from the xml structure for the $node 
 * using requested $lang. If $lang is null then values: Ozone::runData->getLanguage(), 
 * GlobalProperties::$DEFAULT_LANGUAGE are used in that order. If again it does not work
 * the first <text> child node returned.
 * @param SimpleXMLElement $node
 * @return string
 */
function xml_localized_text($node, $lang = null) {
    if ($node == null) {
        return null;
    }
    $runData = Ozone::$runData;
    if ($lang == null) {
        $lang = $runData->getLanguage();
    }
    
    $text = findNodeWithAttribute($node->text, "lang", "$lang");
    if ($text == null) {
        $text = findNodeWithAttribute($node->text, "lang", 
                GlobalProperties::$DEFAULT_LANGUAGE);
    }
    if ($text == null) {
        $text = $node->text[0];
    }
    return "$text";
}

/** 
 * Gets PEAR::Date object set to the current UTC (GMT) time. 
 * Somehow this works for a greater number of configurations 
 * than just calling Date() contstuctor...
 */
function currentDateUTC() {
    return new ODate();
}

function array_copy($array) {
    $newArray = array();
    foreach ($array as $key => $value) {
        $newArray[$key] = $value;
    }
    return $newArray;
}

function print_array($array) {
    foreach ($array as $key => $value) {
        echo "$key => $value\n";
    }
}

function str_replace_once($needle, $replace, $haystack) {
    // Looks for the first occurence of $needle in $haystack
    // and replaces it with $replace.
    $pos = strpos($haystack, $needle);
    if ($pos === false) {
        // Nothing found
        return $haystack;
    }
    return substr_replace($haystack, $replace, $pos, strlen($needle));
}

function microtime_float() {
    list ($usec, $sec) = explode(" ", microtime());
    return ((float) $usec + (float) $sec);
}

function ozone_error_handler($code, $message, $file, $line) {
    throw new PHPErrorException($code, $message, $file, $line);
}

function preg_quote_replacement($string) {
    $out = str_replace('\\', '\\\\', $string);
    $out = preg_replace(';\$([0-9]);', '\\\\$$1', $out);
    return $out;
}

/**
 * Calculates length of an UTF-8 encoded string. The default strlen() fails
 * at this point because it counts bytes rather than characters.
 *
 * @param string $string
 * @return integer
 */
function strlen8($string) {
    return strlen(utf8_decode($string));
}

/**
 * Glues filesystem directory path from multiple components, where each component
 * represent one or more directories, the final can be a file. 
 * E.g. gluePath('/usr', 'bin', 'sed) will produce '/usr/bin/sed'.
 * The result does not have double-slashes nor trailing slash.
 *
 * @param string component1
 * @param string component2
 * @param string ...
 * @return string
 */
function glue_path() {
    $c = func_get_args();
    $first = true;
    foreach ($c as & $co) {
        $co = preg_replace(
                ';[' . preg_quote(
                        DIRECTORY_SEPARATOR) .
                         '/]$;', 
                        '', 
                        $co);
        if (!$first) {
            $co = preg_replace(
                    ';^[' .
                             preg_quote(
                                    DIRECTORY_SEPARATOR) .
                             '/];', 
                            '', 
                            $co);
        } else {
            $first = false;
        }
    }
    return implode(DIRECTORY_SEPARATOR, $c);

}

/**
 * executes a command with the use of a special wrapper that limits time of execution
 *
 * @param string $cmd command to execute
 * @param int $timelimit execution time limit -- default null = no limit
 * @param array $output lines of output
 * @param int $ret_val the return status of the executed command
 */
function exec_time($cmd, $time_limit = null, &$output = null, &$ret_val = null) {
	if ($time_limit == null) {
		$out = exec($cmd, $exec_output, $exec_return_val);
	} else {
		$prog = escapeshellcmd(OZONE_ROOT . "/bin/timelimit.sh");
		$time = (int) $time_limit;
		$newcmd = "$prog $time $cmd";
		$out = exec($newcmd, $exec_output, $exec_return_val);
	}
	if ($output != null) {
		$output = $exec_output;
	}
	if ($ret_val != null) {
		$ret_val = $exec_return_val;
	}
	return $out;
}
