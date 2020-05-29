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
 * @package Wikidot_Web
 * @version $Id: lucene_search.php,v 1.8 2008/12/19 02:13:08 redbeard Exp $
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

require_once ("../php/setup.php");

if (! isset($argv[1])) {
	echo "Usage:\n";
	echo "  php lucene_search.php <search phrase>\n";
	echo "  php lucene_search.php <search phrase> php -- force using the PHP Lucene implementation\n";
	echo "  php lucene_search.php <search phrase> java -- force using the Java Lucene implementation\n";
	exit();
}

if (isset($argv[2]) && $argv[2] == 'java') {
	GlobalProperties::$SEARCH_USE_JAVA = true;
} elseif (isset($argv[2]) && $argv[2] == 'php') {
	GlobalProperties::$SEARCH_USE_JAVA = false;
}

$lucene = new Wikidot_Search_Lucene();
$hits = $lucene->rawQuery($argv[1]);

$i = 0;
echo "indexed: " . $lucene->getCount() . "\n";
echo "hits: " . count($hits) . "\n";

foreach ($hits as $hit) {
	if (++$i == 10) {
		return;
	}
	echo "\n";
	echo $hit;
}
echo "\n";
