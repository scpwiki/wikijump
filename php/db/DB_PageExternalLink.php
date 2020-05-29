<?php
//please extend this class
class DB_PageExternalLink extends DB_PageExternalLinkBase {

	public function buildPageUrl(){
		$page = DB_PagePeer::instance()->selectByPrimaryKey($this->getPageId());
		$site = DB_SitePeer::instance()->selectByPrimaryKey($page->getSiteId());
		
		$h = 'http://'.$site->getDomain().'/'.$page->getUnixName();
		return $h;
	}
}
