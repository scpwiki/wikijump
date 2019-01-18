<?php

class Wikidot_Facade_User extends Wikidot_Facade_Base {
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
		$memberships = DB_MemberPeer::instance()->selectByCriteria($c);
		
		$sites = array();
		foreach ($memberships as $membership) {
			$site = DB_SitePeer::instance()->selectByPrimaryKey($membership->getSiteId());
			if (! $site->getDeleted()) {
				$sites[] = $site;
			}
		}
		return $this->repr($sites);
	}
}
