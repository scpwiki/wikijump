#!/usr/bin/env php
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
 * @category Wikidot
 * @package Wikidot
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */


/**
 * this file creates configuration files that is needed to run Wikidot
 */

$files = array("/files/crontab", "/conf/lighttpd/wikidot_ini.conf", "/conf/secret");

chdir(dirname(__FILE__));
require("../php/setup.php");

chdir(WIKIDOT_ROOT);

$random = random(64);

foreach ($files as $file) {
	$src = WIKIDOT_ROOT."$file.orig";
	$dst = WIKIDOT_ROOT.$file;
	echo "Processing $file .";
	$s = file_get_contents($src);
	echo ".";
	$s = sed($s, $random);
	echo ".";
	file_put_contents($dst, $s);
	echo ".\n";
}

function random($length) {
	$r = "";
	for ($i = 0; $i < $length; $i++) {
		$r .= dechex(rand(0, 15));
	}
	return $r;
}

function sed($s, $random) {
	$s = preg_replace('/%{WIKIDOT:WIKIDOT_ROOT}/', addslashes(WIKIDOT_ROOT), $s);
	$s = preg_replace('/%{WIKIDOT:URL_HOST}/', addslashes(GlobalProperties::$URL_HOST), $s);
	$s = preg_replace('/%{WIKIDOT:URL_HOST_PREG}/', addslashes(GlobalProperties::$URL_HOST_PREG), $s);
	$s = preg_replace('/%{WIKIDOT:URL_DOMAIN}/', addslashes(GlobalProperties::$URL_DOMAIN), $s);
	$s = preg_replace('/%{WIKIDOT:URL_DOMAIN_PREG}/', addslashes(GlobalProperties::$URL_DOMAIN_PREG), $s);
	$s = preg_replace('/%{WIKIDOT:HTTP_PORT}/', addslashes(GlobalProperties::$HTTP_PORT), $s);
	$s = preg_replace('/%{WIKIDOT:RANDOM_STRING}/', $random, $s);
	return $s;
}
