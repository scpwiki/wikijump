<?php

chdir(dirname(__FILE__));
require_once('../php/setup.php');

// map errors to exceptions
function errorHandler($errno, $errstr, $errfile, $errline) {
	if (error_reporting()) {
		throw new Exception($errstr); // internal error not to be mapped to fault
	}
	return true;
}
error_reporting(E_ALL & ~E_NOTICE | E_STRICT);
//set_error_handler('errorHandler', E_ALL & ~E_NOTICE);
ini_set("display_errors", true);

$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey(1);

$page = new Wikidot_Facade_Page($user);
print_r($page->files(array("site" => "www", "page" => "start")));
