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
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

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

$modulePath = WIKIDOT_ROOT . "/php/quickmodules/" . $moduleName . ".php";
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