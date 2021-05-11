<?php

use Ozone\Framework\Database\Database;
use Wikidot\Utils\GlobalProperties;

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
    $q = "SELECT user_karma FROM users WHERE id='".pg_escape_string($userId)."' ";
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

$imgPath = WIKIJUMP_ROOT.'/web/files--common/theme/base/images/karma/';

$imgPath = $imgPath . 'karma_'. $karmaLevel . '.png';

if(file_exists($imgPath)){
    readfile($imgPath);
}
