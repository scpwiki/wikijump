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

class MembersListModule extends SmartyModule {
	
	public function build($runData){
		
		$c = new Criteria();
		$c->add("site_id", $runData->getTemp("site")->getSiteId());
		$c->addJoin("user_id", "ozone_user.user_id");
		
		$pl = $runData->getParameterList();
		$from = $pl->getParameterValue("group", "MODULE");
		$showSince = $pl->getParameterValue("showSince", "MODULE");

		if($showSince == "no" || $showSince == "false" || $showSince == "get lost"){
			$showSince = false;	
		}else{
			$showSince = true;
		}

		if($pl->getParameterType("from") == "MODULE"){
			$from = $pl->getParameterValue("from");	
		}
		if($from !== "admins" && $from !== "moderators"){$from = null;}

		if($from === "admins"){
			$mems = DB_AdminPeer::instance()->select($c);
		}elseif($from === "moderators"){
			$mems = DB_ModeratorPeer::instance()->select($c);
		}else{
			$mems = DB_MemberPeer::instance()->select($c);
		}
		if(count($mems)>0){
			$runData->contextAdd("from", $from);
			$runData->contextAdd("memberships", $mems);	
			$runData->contextAdd("showSince",$showSince);
		}
		
	}
	
}
