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

class ContactsAction extends SmartyAction {
	
	public function isAllowed($runData){
		$userId = $runData->getUserId();
		if($userId == null || $userId <1){
			throw new WDPermissionException(_("Not allowed. You should login first."));
		}
		return true;
	}
	
	public function perform($runData){}
	
	public function addContactEvent($runData){
	
		$pl = $runData->getParameterList();
		
		$targetUserId = $pl->getParameterValue("userId");
		
		$targetUser = DB_OzoneUserPeer::instance()->selectByPrimaryKey($targetUserId);
		
		$user = $runData->getUser();
		
		if($targetUser == null){
			throw new ProcessException(_("User can not be found."), "no_user");	
		}
		
		if($targetUserId == $user->getUserId()){
			throw new ProcessException(_("Is there any point in adding yourself to your contact list? ;-)"), "not_yourself");	
		}
		
		$db = Database::connection();
		$db->begin();
		
		// check if already contacted
		$c = new Criteria();
		$c->add("user_id", $user->getUserId());
		$c->add("target_user_id", $targetUserId);
		
		$contact = DB_ContactPeer::instance()->selectOne($c);
		if($contact){
			throw new ProcessException(_("This user is already in your contacts."),"already_contact");	
		}
		
		// count contacts
		$c = new Criteria();
		$c->add("user_id", $user->getUserId());
		$count = DB_ContactPeer::instance()->selectCount($c);
		if($count>=1000){
			throw new ProcessException(_("Sorry, at this moment you can not add more than 1000 contacts.", "max_reached"));	
		}
		
		//...	
		
		$contact = new DB_Contact();
		$contact->setUserId($user->getUserId());
		$contact->setTargetUserId($targetUserId);
		$contact->save();
		
		$db->commit();
		
	}
	
	public function removeContactEvent($runData){
		$pl = $runData->getParameterList();
		$user = $runData->getUser();
		$targetUserId = $pl->getParameterValue("userId");
		
		if($targetUserId == null){
			throw new ProcessException(_("No user found."), "no_user");	
		}
		
		$c = new Criteria();
		$c->add("user_id", $user->getUserId());
		$c->add("target_user_id", $targetUserId);
		
		DB_ContactPeer::instance()->delete($c);
		
	}
	
}
