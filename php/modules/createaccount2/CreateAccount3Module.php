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

class CreateAccount3Module extends SmartyModule {

	public function build($runData){
		$user = $runData->getUser();
		if(!$user){
			throw new ProcessException(_('No valid user found - account creation failed.'));
		}
		$runData->contextAdd("user", $user);
		$pl = $runData->getParameterList();
		
		$originalUrl = $pl->getParameterValue('origUrl');
		$runData->contextAdd('originalUrl', $originalUrl);
		$runData->contextAdd('originalUrlStripped', preg_replace(';^https?://;', '', $originalUrl));
		
	}

}
