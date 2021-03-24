<?php

use Wikidot\Utils\GlobalProperties;

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

$lucene = new Wikidot\Search\Lucene();
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
