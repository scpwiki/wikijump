<?php 
//please extend this class
class DB_ApiKeyPeer extends DB_ApiKeyPeerBase {
	static public function getUserByKey($key) {
		$user = null;
		$api_key = self::instance()->selectByPrimaryKey($key);
		if ($api_key) {
			$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($api_key->getUserId());
		}
		return $user;
	}
}

