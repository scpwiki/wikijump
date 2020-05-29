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
 * @package Wikidot
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

class MembershipApplyModule extends SmartyModule{
	
	public function build($runData){
		$site = $runData->getTemp("site");
		$userId = $runData->getUserId();
		
		$reason = null;
		if(!$runData->isUserAuthenticated()){
			$reason = "not_logged";
		}	
		
		$settings = $site->getSettings();
		
		if(!$settings->getAllowMembershipByApply()){
			$reason = "not_enabled";
			$runData->contextAdd("reason", $reason);
			return;
		}
		
		// check if not a member already
		$c = new Criteria();
		$c->add("site_id", $site->getSiteId());
		$c->add("user_id", $userId);
		$a = DB_MemberPeer::instance()->selectOne($c);
		if($a != null){
			$reason = "already_member";
			$runData->contextAdd("reason", $reason);
			return;
		}
		
		// see if there is already an application...
		$c = new Criteria();
		$c->add("site_id", $site->getSiteId());
		$c->add("user_id", $userId);
		$a = DB_MemberApplicationPeer::instance()->selectOne($c);
		if($a != null){
			$reason = "already_applied";
			$runData->contextAdd("reason", $reason);
			return;
		}

		if($reason !== null){
			$runData->contextAdd("reason", $reason);	
		}
	}
	
}
