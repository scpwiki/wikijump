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

class ManageSuperUserModule extends SmartyModule {
	
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
	
	public function build($runData){
		
		$pl = $runData->getParameterList();
		
		$o = DB_OzoneUserPeer::instance()->selectByPrimaryKey(1);
		$u = array(
			"nick_name" => $o->getNickName(),
		);
		$runData->contextAdd("user", $u);
		
		if ($key = $pl->getParameterValue("key")) {
			$runData->contextAdd("key", $key);
		}
		
	}
	
}
