<?php


namespace Wikidot\Facade;

use Wikidot\Facade\Base;
use \WDPermissionException;
use Criteria;
use DB\MemberPeer;
use DB\SitePeer;



class User extends Base {
	/**
	 * Just check if user exists and has access to the API.
	 * 
	 * @param struct $args
	 * @return struct
	 */
	public function valid($args) {
		$this->parseArgs($args, array("performer"));
	}
	
	/**
	 * Get sites of a user. This is a fake one!
	 * 
	 * @param struct $args
	 * @return struct
	 */
	public function sites($args) {
		$this->parseArgs($args, array("performer", "user"));
		
		if ($this->performer->getUserId() != $this->user->getUserId()) {
			throw new WDPermissionException("One can only list their own sites");
		}
		
		$c = new Criteria();
		$c->add("user_id", $this->user->getUserId());
		$memberships = MemberPeer::instance()->selectByCriteria($c);
		
		$sites = array();
		foreach ($memberships as $membership) {
			$site = SitePeer::instance()->selectByPrimaryKey($membership->getSiteId());
			if (! $site->getDeleted()) {
				$sites[] = $site;
			}
		}
		return $this->repr($sites);
	}
}
