<?php

/**
 * Wikidot (Community Edition) - free wiki collaboration software
 * 
 * 							http://www.wikidot.org
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
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 * 
 * @category Wikidot
 * @package Wikidot_Tools
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc. (http://www.wikidot-inc.com)
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

chdir(dirname(__FILE__)); // unifies CLI/CGI cwd handling
require ('../php/setup.php');

define('SMARTY_DIR', PathManager :: smartyDir());
require_once (SMARTY_DIR.'Smarty.class.php');

$logger = OzoneLogger::instance();
$loggerFileOutput = new OzoneLoggerFileOutput();
$loggerFileOutput->setLogFileName(WIKIDOT_ROOT."/logs/om-generation.log");
$logger->addLoggerOutput($loggerFileOutput);
$logger->setDebugLevel("debug");

$logger->debug("request processing started, logger initialized");

if(in_array('--drop-tables', $argv)){
	$dropTables = true;
} else {
	$dropTables = false;
}
$executeSql = true;
if(in_array('-o', $argv)){
	// output to a file
	$po = array_search('-o', $argv);
	$ofile = $argv[$po + 1];
	$executeSql = false;	
}

$schemaFiles = ls(WIKIDOT_ROOT."/conf/database", "*-db.xml");

if (sizeof($schemaFiles) == 0) {
	die("Error: no database schema files found\n");
}

// connect to the database
Database::init();
$db = Database::connection();

$db->begin();

echo ("database name: ".GlobalProperties :: $DATABASE_NAME."\n");


$database = new DBGeneratorDatabase();
$database->setExecuteSql($executeSql);

foreach ($schemaFiles as $key => $file) {
	echo "----------------------------------------\n";
	echo "processing file $file:\n";
	echo "----------------------------------------\n";
	$xml = simplexml_load_file(WIKIDOT_ROOT."/conf/database/$file");
	
	$database->addSchema($xml);
}
// update references between tables
$database->updateReferences();

$database->executeSQL();
echo "generating classes\n";
$database->generateClasses();

$sql = $database->getSql();

if($ofile){
	file_put_contents($ofile, implode(";\n", $sql));
}

$db->commit();

