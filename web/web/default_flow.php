<?php

use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\WDDefaultFlowController;

require ('../php/setup.php');

// to avoid caching
header("Cache-Control: no-cache, must-revalidate"); // HTTP/1.1
header("Expires: Mon, 26 Jul 1997 05:00:00 GMT"); // Date in the past

try {
    $controller = new WDDefaultFlowController();
    $controller->process();
} catch (Exception $e) {
    echo $e->getMessage();
}
