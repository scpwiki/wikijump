<?php 

namespace DB;

use DB_ApiKeyPeerBase;
use DB\OzoneUserPeer;



//please extend this class
class ApiKeyPeer extends DB_ApiKeyPeerBase {
	static public function getUserByKey($key) {
		$user = null;
		$api_key = self::instance()->selectByPrimaryKey($key);
		if ($api_key) {
			$user = OzoneUserPeer::instance()->selectByPrimaryKey($api_key->getUserId());
		}
		return $user;
	}
}

