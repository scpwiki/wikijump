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

header('Content-type: image/png');
$offset = 3600 * 1;	
// calc the string in GMT not localtime and add the offset
$expire = "Expires: " . gmdate("D, d M Y H:i:s", time() + $offset) . " GMT";
//output the HTTP header
Header($expire);
$gmt_mtime = gmdate('D, d M Y H:i:s', time() ) . ' GMT';
header("Last-Modified: " . $gmt_mtime );
header('Cache-Control: max-age=3600, must-revalidate');

/* Do everything here. */

$u = $_SERVER['REQUEST_URI'];
$split = explode("/", $u);
$parms = array();
for($i=1; $i<count($split); $i+=2){
    $parms[$splited[$i]] = $splited[$i+1];

}

if(!isset($parms['u'])){
    if(!isset($_GET['u'])){
        return;
    }else{
        $userId = $_GET['u'];
    }
}else{
    $userId = $parms['u'];
}
if(!is_numeric($userId)){
    return;
}
$userId += 0;
if(!is_int($userId) || $userId <=0){
    return;
}

$karmaLevel = false;
if(GlobalProperties::$USE_MEMCACHE == true){
	$memcache = new Memcache();
	$memcache->connect(GlobalProperties::$MEMCACHE_HOST, GlobalProperties::$MEMCACHE_PORT);
	
	/* Check memcache for the karma level. */
	$key = 'user_karma_level..'.$userId;
	$karmaLevel = $memcache->get($key);
}
if(is_bool($karmaLevel) && !$karmaLevel){
    Database::init();
    /* Get karma level. */
    $q = "SELECT * FROM user_karma WHERE user_id='".pg_escape_string($userId)."' ";
    $db = Database::$connection;
    $r = $db->query($q);
    $row = $r->nextRow();
    $karmaLevel = 0;
    if($row){
        $karmaLevel = $row['level'];
    }
    if(isset($key)){
        $memcache->set($key, $karmaLevel, 0, 3600);
    }
}

$imgPath = WIKIDOT_ROOT.'/web/files--common/theme/base/images/karma/';

$imgPath = $imgPath . 'karma_'. $karmaLevel . '.png';

if(file_exists($imgPath)){
    readfile($imgPath);
}