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

class CreateAccount2Module extends SmartyModule {
	
	public function isAllowed($runData){
		if($runData->getUserId() !== null){
			throw new ProcessException(_("You are already logged in. Why would you want to create a new account?"), "logged_in"); 	
		}	
		$rstep = $runData->sessionGet("rstep");
		return true;	
		
	}
	
	public function build($runData){
		
		$runData->contextAdd('evcode', $runData->sessionGet('evcode'));
		$runData->sessionAdd("rstep", 2);
		
		$data = $runData->sessionGet("ca_data");
		$email = $data['email'];
		$name = $data['name'];
		
		$runData->contextAdd("email", $email);
		$runData->contextAdd("name", $name);
	}
	
}
