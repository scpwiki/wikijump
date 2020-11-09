<?php
require_once('../php/setup.php');
require_once('Zend/XmlRpc/Server.php');

$server = new Zend_XmlRpc_Server();
$server->setClass('PingBackServer', 'pingback');

echo $server->handle();
