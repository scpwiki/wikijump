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

class ManageSuperUserAction extends SmartyAction {
	
	public function isAllowed($runData){
		$pl = $runData->getParameterList();
		if ($key = $pl->getParameterValue("key")) {
			if (GlobalProperties::$SECRET_MANAGE_SUPERADMIN == $key) {
				return true;
			}
		}
		WDPermissionManager::instance()->hasPermission('manage_site', $runData->getUser(), $runData->getTemp("site"));	
		return true;
	}
	
	public function perform($r){}
	
	public function saveEvent($runData){
		$pl = $runData->getParameterList();
		
		$nick_name = $pl->getParameterValue("nick_name");
		$password = $pl->getParameterValue("password1");
		
		$u = DB_OzoneUserPeer::instance()->selectByPrimaryKey(1);
		$u->setName($nick_name);
		$u->setEmail($nick_name);
		$u->setNickName($nick_name);
		$u->setUnixName(WDStringUtils::toUnixName($nick_name));
		$u->setPassword(md5($password));
		$u->setSuperAdmin(true);
		
		$u->save();
	}
	
}
