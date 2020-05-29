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

abstract class LoggedSecureModule extends SmartyModule {
	
	public function isAllowed($runData){
		if($runData->isUserAuthenticated()){
			$runData->contextAdd("loggedUserId", $runData->getUserId());
			return true;
		}	else {
			
			// if some conditions are met, you can be succesfuly redirected after the login
			if($runData->getRequestMethod() == "GET" || $runData->getAction === null){
				$runData->contextAdd("requestedTemplate", $runData->getScreenTemplate());
				$rp = $runData->getParameterList()->asArray();
				unset($rp['template']);
				$runData->contextAdd("requestedParameters", serialize($rp));
			}
			return false;
				
		}
		
	}
	
}
