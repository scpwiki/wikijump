#!/usr/bin/env php
<?php

use Wikidot\Facade\Forum;
use Wikidot\Facade\Page;
use Wikidot\Facade\Site;
use Wikidot\Facade\User;

chdir(dirname(__FILE__));
require_once('../php/setup.php');

// map errors to exceptions
function errorHandler($errno, $errstr, $errfile, $errline) {
	if (error_reporting()) {
		throw new Exception($errstr); // internal error not to be mapped to fault
	}
	return true;
}
error_reporting(E_ALL & ~E_NOTICE);
set_error_handler('errorHandler', E_ALL & ~E_NOTICE);

// construct facade objects
$server = new Zend_XmlRpc_Server();
$server->setClass(new User(), 'user');
$server->setClass(new Site(), 'site');
$server->setClass(new Page(), 'page');
$server->setClass(new Forum(), 'forum');
Zend_XmlRpc_Server_Cache::save('/tmp/xmlrpcapi.cache', $server);

// map Wikijump\Facade_Exception to XML-RPC faults
Zend_XmlRpc_Server_Fault::attachFaultException('Wikijump\Facade\Exception');
Zend_XmlRpc_Server_Fault::attachFaultException('WDPermissionException');

// run XML-RPC server
echo $server->handle(new Zend_XmlRpc_Request_Stdin());
