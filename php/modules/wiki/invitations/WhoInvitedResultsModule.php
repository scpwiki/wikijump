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

class WhoInvitedResultsModule extends SmartyModule {
	
	public function build($runData){
		
		$site = $runData->getTemp("site");
		
		$pl = $runData->getParameterList();
		
		$userId = $pl->getParameterValue("userId");
		$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($userId);
		
		if(!$user){
			throw new ProcessException(_("Invalid user."));	
		}
		$c = new Criteria();
		$c->add("user_id", $userId);
		$c->add("site_id", $site->getSiteId());
		$mem = DB_MemberPeer::instance()->selectOne($c);
		
		if(!$mem){
			throw new ProcessException(_("The user is not a Member of this Wiki."));	
		}
		
		$link = DB_MembershipLinkPeer::instance()->selectByUserId($site->getSiteId(), $userId);
		if(!$link){
			$runData->contextAdd("noData", true);	
		}else{

			$chain = array();
			$chain[] = array('user' => $user, 'link' => $link);
			if($link->getByUserId()){
				do{
					
					// get "parent"
					// get link for the user
					$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($link->getByUserId());
					$link = DB_MembershipLinkPeer::instance()->selectByUserId($site->getSiteId(), $user->getUserId());
					$chain[] = array('user' => $user, 'link' => $link);
				}while($user && $link && $link->getByUserId());
			}
			$runData->contextAdd("chain", array_reverse($chain)); 
		}
			
	}
	
}
