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

class MembershipEmailInvitationModule extends SmartyModule {
	
	public function build($runData){
		$pl = $runData->getParameterList();
		$user = $runData->getUser();
		$hash = $pl->getParameterValue("hash");

		// get the invitation entry (if any)
		
		$c = new Criteria();
		$c->add("hash", $hash);
		$c->add("accepted", false);
		
		$inv = DB_EmailInvitationPeer::instance()->selectOne($c);
		
		$runData->contextAdd("user", $user);
		
		if(!$inv){
			//sorry, no invitation
			return;
		}
		
		$site = DB_SitePeer::instance()->selectByPrimaryKey($inv->getSiteId());
		
		$sender = DB_OzoneUserPeer::instance()->selectByPrimaryKey($inv->getUserId());
		$runData->contextAdd("sender", $sender);
		$runData->contextAdd("site", $site);
		$runData->contextAdd("invitation", $inv);
		$runData->contextAdd("hash", $hash);
			
	}
	
}
