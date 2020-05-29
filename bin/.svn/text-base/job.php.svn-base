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

// initialize things now
	
$logger = OzoneLogger::instance();
$loggerFileOutput = new OzoneLoggerFileOutput();
$loggerFileOutput->setLogFileName(WIKIDOT_ROOT."/logs/jobs.log");
$logger->addLoggerOutput($loggerFileOutput);
$logger->setDebugLevel("debug");

$logger->debug("request processing started, logger initialized");
	
// initialize OZONE object too
Ozone::init();
$runData = new RunData();
$runData->init();
Ozone::setRunData($runData);

// Set the text domain as 'messages'
$gdomain = 'messages';
bindtextdomain($gdomain, WIKIDOT_ROOT.'/locale'); 
textdomain($gdomain);


$jobName = $argv[1];

$classFile = WIKIDOT_ROOT.'/php/jobs/'.$jobName.'.php';

require_once $classFile;
	
$job = new $jobName();

$job->run();
