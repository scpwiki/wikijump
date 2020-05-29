<?php
//please extend this class
class DB_Ucookie extends DB_UcookieBase {

	public function generate(DB_Site $site, DB_OzoneSession $session) {
		$key = md5(gmdate("c") . rand());
		
		$this->setSite($site);
		$this->setOzoneSession($session);
		$this->setUcookieId($key);
	}
}
