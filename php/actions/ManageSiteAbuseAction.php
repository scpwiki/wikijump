<?php
/**
 * Wikidot - free wiki collaboration software
 * http://www.wikidot.org
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
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 * 
 * @category Wikidot
 * @package Wikidot
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

class ManageSiteAbuseAction extends SmartyAction {
	
	public function isAllowed($runData){
		WDPermissionManager::instance()->hasPermission('manage_site', $runData->getUser(), $runData->getTemp("site"));	
		return true;
	}
	
	public function perform($r){}
	
	public function clearPageFlagsEvent($runData){
		$site = $runData->getTemp("site");
		$pl = $runData->getParameterList();
		
		$path = $pl->getParameterValue("path");
		
		if($path == null || $path == ''){
			throw new ProcessException(_("Error processing the request. No page specified"), "no_path");	
		}	
		
		$q = "UPDATE page_abuse_flag SET site_valid=FALSE WHERE " .
				"site_id='".$site->getSiteId()."' " .
				"AND path='".db_escape_string($path)."' " .
				"AND site_valid=TRUE";
		
		$db = Database::connection();
		$db->query($q);
		
	}
	
	public function clearUserFlagsEvent($runData){
		$site = $runData->getTemp("site");
		$pl = $runData->getParameterList();
		
		$targetUserId = $pl->getParameterValue("userId");
		$targetUser = DB_OzoneUserPeer::instance()->selectByPrimaryKey($targetUserId);

		if($targetUser == null){
			throw new ProcessException(_("Error processing the request. No user found."), "no_user");	
		}	
		
		$q = "UPDATE user_abuse_flag SET site_valid=FALSE WHERE " .
				"site_id='".$site->getSiteId()."' " .
				"AND target_user_id='".$targetUser->getUserId()."' " .
				"AND site_valid=TRUE";
		
		$db = Database::connection();
		$db->query($q);

	}
	
	public function clearAnonymousFlagsEvent($runData){
		$site = $runData->getTemp("site");
		$pl = $runData->getParameterList();
		
		$address = $pl->getParameterValue("address");
		$proxy = $pl->getParameterValue("proxy");
		if($proxy){
			$proxy = "TRUE";
		}else{
			$proxy = "FALSE";
		}
		
		if(preg_match('/^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+/', $address) !==1){
			throw new ProcessException(_("Wrong address format."), "bad_address");
		}

		$q = "UPDATE anonymous_abuse_flag SET site_valid=FALSE WHERE " .
				"site_id='".$site->getSiteId()."' " .
				"AND address='$address' " .
				"AND proxy=$proxy ".
				"AND site_valid=TRUE";
		
		$db = Database::connection();
		$db->query($q);

	}
	
}
