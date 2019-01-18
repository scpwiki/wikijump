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

class ManageUsersModule extends SmartyModule {
	
	public function isAllowed($runData){
		if ($runData->getTemp("site")->getSiteId() != 1) {
			throw new WDPermissionException("No permission");
		}
		WDPermissionManager::instance()->hasPermission('manage_site', $runData->getUser(), $runData->getTemp("site"));
		
		return true;
	}
	
	public function build($runData){
		
		$users = array();
		$c = new Criteria();
		$c->add('user_id', '1', '>');

		foreach (DB_OzoneUserPeer::instance()->select($c) as $user) {
            $admin = WDPermissionManager::hasPermission('manage_site', $user, 1) ? 1 : 0;
            $mod = WDPermissionManager::hasPermission('moderate_site', $user, 1) ? 1 : 0;

			$users[] = array(
				"nick_name" => $user->getNickName(),
				"user_id" => $user->getUserId(),
                "mod" => $mod,
                "admin" => $admin,
			);
		}
		for ($i = 0; $i < 5; $i++) {
			$users[] = array("user_id" => "new$i");
		}
		$runData->contextAdd("users", $users);
		
	}
	
}
