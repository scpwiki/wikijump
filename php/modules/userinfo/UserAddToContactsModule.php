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

class UserAddToContactsModule extends SmartyLocalizedModule {
	
	public function isAllowed($runData){
		$userId = $runData->getUserId();
		if($userId == null || $userId <1){
			throw new WDPermissionException(_("You should login first."));
		}
		return true;
	}
	
	public function build($runData){
		$pl = $runData->getParameterList();
		
		$targetUserId = $pl->getParameterValue("userId");
		
		$targetUser = DB_OzoneUserPeer::instance()->selectByPrimaryKey($targetUserId);
		
		if($targetUser == null){
			throw new ProcessException(_("User can not be found."), "no_user");	
		}
		
		// check how many contacts so far...

		$runData->contextAdd("user", $targetUser);	
	}
	
}
