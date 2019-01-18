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
 * @package Ozone
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

$applicationDir = $argv[1];
echo "The application dir is: " .$applicationDir."\n";

require_once ($applicationDir."/conf/GlobalProperties.php");
require_once ("../php/core/autoload.inc.php");
require_once ("../php/core/functions.php");

$tz = GlobalProperties::$SERVER_TIMEZONE;
putenv("TZ=$tz");
require_once('/usr/lib/php/Date.php');
require_once('/usr/lib/php/Date/TimeZone.php');
require_once('/usr/lib/php/Date/Span.php');

define('SMARTY_DIR', PathManager :: smartyDir());
require_once (SMARTY_DIR.'Smarty.class.php');

// connect to the database
Database::init();
$db = Database::connection();

$scheduler = new Scheduler();

$scheduler->setClassPath($applicationDir."/php/jobs");

$schedulerFiles = ls($applicationDir."/conf/scheduler", "*-jobs.xml");

foreach ($schedulerFiles as $key => $file) {
	echo "----------------------------------------\n";
	echo "processing file $file:\n";
	echo "----------------------------------------\n";
	$xml = simplexml_load_file($applicationDir."/conf/scheduler/$file");
	
	##$database = new DBDatabase($xml);
	$scheduler->parseConfigXml($xml);

}

$scheduler->start();
