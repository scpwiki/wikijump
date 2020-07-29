<?php
require_once ("../php/setup.php");

$lucene = new Wikijump\Search\Lucene();
$lucene->createIndex();
$lucene->indexAllSitesVerbose();
