<?php

namespace Ozone\Framework;


// define autoload paths
use Wikidot\Utils\GlobalProperties;

$opath = OZONE_ROOT;
$apath = WIKIJUMP_ROOT;

$classpath = array();

//$classpath[] = $opath."/php/core/Database";
//$classpath[] = $opath."/php/core";
//$classpath[] = $opath."/php/core/Exceptions";
//$classpath[] = $apath."/php/Utils";
//$classpath[] = $apath."/php/DB";
//$classpath[] = $apath."/php/pingback";
$classpath[] = $apath."/conf";

$GLOBALS['classpath'] = $classpath;

$paths = explode(PATH_SEPARATOR, get_include_path());
$paths = array_merge($paths, $classpath);
$paths = array_unique($paths);
$paths = implode(PATH_SEPARATOR, $paths);
set_include_path($paths);

/**
 * Function responsible for including .php files containing Class definitions.
 * @param string $className name of the Class
 */
/* spl_autoload_register( function($className) {
	trigger_error("Autoloading ".$className);
	$className = str_replace('\\','/', $className);
	include_once($className.'.php');
	$class_actual = explode('/',$className);
	if(! class_exists(end($class_actual)) && ! interface_exists(end($class_actual))) {
		trigger_error("Class $className not loaded.");
	}
	else { trigger_error("Loaded ".end($class_actual)); }
	return;
}); */

spl_autoload_register(function ($class) {
    if(GlobalProperties::$LOGGER_LEVEL == "debug") {
        trigger_error("Paths: " . get_include_path());
    }
            $file = str_replace('\\', DIRECTORY_SEPARATOR, $class).'.php';
            if (stream_resolve_include_path($file)) {
		    require $file;
		    if(GlobalProperties::$LOGGER_LEVEL == "debug") {
                trigger_error("Loaded $file for $class");
            }
                return true;
	    }
//	    trigger_error("Failed to load $file for $Class");
            return false;
        });
