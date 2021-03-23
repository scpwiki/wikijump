<?php
require_once ("../php/setup.php");

$lucene = new Wikidot\Search\Lucene();
$lucene->createIndex();
$lucene->indexAllSitesVerbose();
