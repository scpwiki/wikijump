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

$files = array("/web/.htaccess", "/files/apache.vhost.wikidot.conf", "/files/crontab", "/files/lighttpd-wikidot.conf");

chdir(dirname(__FILE__));
require("../php/setup.php");

chdir(WIKIDOT_ROOT);

foreach ($files as $file) {
	$src = WIKIDOT_ROOT."$file.orig";
	$dst = WIKIDOT_ROOT.$file;
	echo "Processing $file .";
	$s = file_get_contents($src);
	echo ".";
	$s = sed($s);
	echo ".";
	file_put_contents($dst, $s);
	echo ".\n";
}

// generate RSA key

echo "Generating RSA keys ..\n";
exec('sh bin/generate_keys.sh');

echo "Done!!!\n";

echo <<<EOT

Now please use the following files:

files/apache.vhost.wikidot.conf - append to your Apache configuration,
OR files/lighttpd-wikidot.conf - include to lighttpd configuration (sample Lighttpd configuration: files/lighttpd.conf)
files/crontab - append to your Crontab configuration

You might also get a Flickr API key from http://flickr.com and
put it to files/flickr-api-key.txt


EOT;


function sed($s) {
	$s = preg_replace('/%{WIKIDOT:WIKIDOT_ROOT}/', WIKIDOT_ROOT, $s);
	$s = preg_replace('/%{WIKIDOT:URL_HOST}/', GlobalProperties::$URL_HOST, $s);
	$s = preg_replace('/%{WIKIDOT:URL_HOST_PREG}/', GlobalProperties::$URL_HOST_PREG, $s);
	$s = preg_replace('/%{WIKIDOT:URL_DOMAIN}/', GlobalProperties::$URL_DOMAIN, $s);
	$s = preg_replace('/%{WIKIDOT:URL_DOMAIN_PREG}/', GlobalProperties::$URL_DOMAIN_PREG, $s);
	$s = preg_replace('/%{WIKIDOT:SUPPORT_EMAIL}/', GlobalProperties::$SUPPORT_EMAIL, $s);
	return $s;
}
