<?php


namespace DB;

use GlobalProperties;


//please extend this class
class PageExternalLink extends PageExternalLinkBase {

	public function buildPageUrl(){
		$page = PagePeer::instance()->selectByPrimaryKey($this->getPageId());
		$site = SitePeer::instance()->selectByPrimaryKey($page->getSiteId());

		$h = GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain().'/'.$page->getUnixName();
		return $h;
	}
}
