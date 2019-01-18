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

class NewWUsersModule extends SmartyModule {
	
	public function render($runData){
		$key = "module..0..NewWUsersModule";
		$mc = OZONE::$memcache;
		
		$out = $mc->get($key);
		if(!$out){
			$out = parent::render($runData);
			$mc->set($key, $out, 0, 180);	
		} 
		
		return $out;
		
	}
	
	public function build($runData){
		// get a few new users
		
		$c = new Criteria();
		$c->add('user_id', 0, '>');
		$c->addOrderDescending("user_id");
		
		$c->setLimit(5);
		
		$users = DB_OzoneUserPeer::instance()->select($c);
		
		$runData->contextAdd("users", $users);
			
	}
	
}
