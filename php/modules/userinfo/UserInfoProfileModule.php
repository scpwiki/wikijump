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

class UserInfoProfileModule extends SmartyLocalizedModule {
	
	public function build($runData){
		$pl = $runData->getParameterList();
		$userId = $pl->getParameterValue("user_id");
		
		$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($userId);
		$runData->contextAdd("user",$user);
		
		$avatarUri = '/common--images/avatars/'.floor($userId/1000).'/'.$userId.'/a48.png';
		$runData->contextAdd("avatarUri", $avatarUri);
		
		// get profile page to include
		$pageName = "profile:".$user->getUnixName();
		
		$c = new Criteria();
		$c->add("unix_name", "profiles");
		$site = DB_SitePeer::instance()->selectOne($c);
		
		$page = DB_PagePeer::instance()->selectByName($site->getSiteId(), $pageName);
		
		if($page !== null){
		
			$compiled = $page->getCompiled();
			$runData->contextAdd("profileContent", $compiled);
			$runData->contextAdd("wikiPage", $page);
		}
		$runData->contextAdd('karmaLevel', $user->getKarmaLevel());
	}
	
}
