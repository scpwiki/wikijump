<?php
require ('../php/setup.php');

// to avoid caching
header("Cache-Control: no-cache, must-revalidate"); // HTTP/1.1
header("Expires: Mon, 26 Jul 1997 05:00:00 GMT"); // Date in the past


// all the parameters are stored in the POST body.
$data = file_get_contents('php://input');

if ($data != null && $data !== '') {
    $json = new JSONService(SERVICES_JSON_LOOSE_TYPE);
    $parsedData = $json->decode($data);
}

// find quickmodule name
$moduleName = $_GET['module'];
// check if exists

$modulePath = WIKIJUMP_ROOT . "/php/quickmodules/" . $moduleName . ".php";
if (file_exists($modulePath)) {
    require_once ($modulePath);

    $module = new $moduleName();
    $response = $module->process($parsedData);

    if ($parsedData['callbackIndex'] !== null) {
        $response['callbackIndex'] = $parsedData['callbackIndex'];
    }
    if ($response != null) {
        if (!$json) {
            $json = new JSONService(SERVICES_JSON_LOOSE_TYPE);
        }
        echo $json->encode($response);
    }
} else {
    return;
}

/*
 * example query:
 * http://www.example.com/quickmodule.php?module=PageLookupQModule&q=howto&s=1
 */
