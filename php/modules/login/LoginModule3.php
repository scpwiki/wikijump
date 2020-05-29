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

class LoginModule3 extends SmartyModule {
	
	public function build($runData){
		$pl = $runData->getParameterList();
		
		$backUrl = $pl->getParameterValue('backUrl');
		$runData->contextAdd('backUrl', $backUrl);
		
		// check if reset remebered user
		$pl = $runData->getParameterList();

		if($pl->getParameterValue("reset")){
			
			setcookie('welcome', 'dummy', time() - 10000000, "/", GlobalProperties::$SESSION_COOKIE_DOMAIN);
		}else{
			// check if a recognized user
			
			$userId = $_COOKIE['welcome'];
			if($userId && is_numeric($userId) && $userId >0){
				$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($userId);
			}
			if($user == null){
				setcookie('welcome', 'dummy', time() - 10000000, "/", GlobalProperties::$SESSION_COOKIE_DOMAIN);
			}
		}
		
		$runData->contextAdd("user", $user);
		
	}
	
}
