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

class MembershipApplyAction extends SmartyAction {
	
	public function perform($r){}
	
	public function applyEvent($runData){

		$site = $runData->getTemp("site");
		$pl = $runData->getParameterList();
		$comment = trim($pl->getParameterValue("comment"));
		$userId = $runData->getUserId();
		
		$user = $runData->getUser();
		if($user == null){
			throw new ProcessException(_("Sorry, you are not logged in. Anonymous users can not apply ;-)"));	
		}
		// check for permissions
			WDPermissionManager::instance()->hasPermission("become_member", $user, $site);

		if($comment == null || $comment==''){
			throw new ProcessException(_("You should write something in the box..."), "no_text");
		}
		
		$db = Database::connection();
		$db->begin();
		
		$settings = $site->getSettings();
		
		if(!$settings->getAllowMembershipByApply()){
			throw new ProcessException(_("Applying is disabled for this site."), 'not_enabled');
		}

		// see if there is already an application...
		$c = new Criteria();
		$c->add("site_id", $site->getSiteId());
		$c->add("user_id", $userId);
		$a = DB_MemberApplicationPeer::instance()->selectOne($c);
		if($a != null){
			// application already exists!!!
			throw new ProcessException(_("You have already applied to this site"), "already_applied");
		}
		
		// check if not a member already
		$a = DB_MemberPeer::instance()->selectOne($c);
		if($a != null){
			throw new ProcessException( _("You already are a member of this site."), "already_member");
			$db->commit();
			return;	
		}
		
		$application = new DB_MemberApplication();
		$application->setSiteId($site->getSiteId());
		$application->setUserId($userId);
		$application->setDate(new ODate());
		
		$application->setComment($comment);
		
		$application->save();
		
		AdminNotificationMaker::instance()->newMemberApplication($application);
		
		$db->commit();
			
	}
	
	public function applyByPasswordEvent($runData){

		$site = $runData->getTemp("site");
		$pl = $runData->getParameterList();
		$comment = $pl->getParameterValue("comment");
		$userId = $runData->getUserId();
		
		$settings = $site->getSettings();
		
		$db = Database::connection();
		$db->begin();
		
		if(!$settings->getAllowMembershipByPassword()){
			throw new ProcessException(_("Applying is disabled for this site."), "not_enabled");
		}
		
		$user = $runData->getUser();
		if($user == null){
			throw new ProcessException(_("Sorry, you are not logged in. Anonymous users can not apply ;-)"));	
		}
		// check for permissions
		WDPermissionManager::instance()->hasPermission("become_member", $user, $site);
			
		$c = new Criteria();
		$c->add("site_id", $site->getSiteId());
		$c->add("user_id", $userId);
		
		$a = DB_MemberPeer::instance()->selectOne($c);
		if($a != null){
			$runData->ajaxResponseAdd('status', 'already_member');
			$runData->ajaxResponseAdd("message", _("You already are a member of this site."));
			$db->commit();
			return;	
		}
		
		$password = $pl->getParameterValue("password");
		if($password === $settings->getMembershipPassword()){
			// create member...
			
			// check if not >=10 members
			if($site->getPrivate()){
				$settings = $site->getSettings();
				$maxMembers = $settings->getMaxPrivateMembers();
				$c = new Criteria();
				$c->add("site_id", $site->getSiteId());
				$cmem = DB_MemberPeer::instance()->selectCount($c);
				if($cmem >= $maxMembers){
					throw new ProcessException(sprintf(_('Sorry, at the moment max %d member limit apply for private Wikis. The Site would have to be upgraded to allow more members.'), $maxMembers));	
				}
			}
			
			$mem = new DB_Member();
			$mem->setUserId($userId);
			$mem->setSiteId($site->getSiteId());
			$mem->setDateJoined(new ODate());
			$mem->save();
			
			$ml = new DB_MembershipLink();
			$ml->setUserId($userId);
			$ml->setSiteId($site->getSiteId());
			$ml->setDate(new ODate());
			$ml->setType('BY_PASSWORD');
			$ml->save();
		
			$runData->ajaxResponseAdd("message", _("Congratulations! You are now a member of this site!"));
			
			// remove application (if any) and invitations
			$c = new Criteria();
			$c->add("site_id", $site->getSiteId());
			$c->add("user_id", $userId);
			
			DB_MemberApplicationPeer::instance()->delete($c);
			DB_MemberInvitationPeer::instance()->delete($c);
			
			AdminNotificationMaker::instance()->newMemberByPassword($site, $user);
			
		} else {
			$runData->ajaxResponseAdd('status', 'wrong_password');
			$runData->ajaxResponseAdd("message", _("Sorry, wrong password..."));
			$db->commit();
			return;		
		}
		
		$db->commit();
	}
	
	public function acceptEmailInvitationEvent($runData){
		$pl = $runData->getParameterList();
		$user = $runData->getUser();
		$hash = $pl->getParameterValue("hash");

		// get the invitation entry (if any)
		
		$c = new Criteria();
		$c->add("hash", $hash);
		$c->add("accepted", false);
		
		$inv = DB_EmailInvitationPeer::instance()->selectOne($c);
		
		$runData->contextAdd("user", $user);
		
		if(!$inv){
			throw new ProcessException(_("Sorry, no invitation can be found."));
		}
		$site = DB_SitePeer::instance()->selectByPrimaryKey($inv->getSiteId());
		
		// check if not a member already
		
		$c = new Criteria();
		$c->add("user_id", $user->getUserId());
		$c->add("site_id", $site->getSiteId());
		
		$mem = DB_MemberPeer::instance()->selectOne($c);
		
		if($mem){
			throw new ProcessException(_("It seems you already are a member of this site! Congratulations anyway ;-)"));	
		}
		
		// check if not > max _members
		if($site->getPrivate()){
			$settings = $site->getSettings();
			$maxMembers = $settings->getMaxPrivateMembers();
			$c = new Criteria();
			$c->add("site_id", $site->getSiteId());
			$cmem = DB_MemberPeer::instance()->selectCount($c);
			if($cmem >= $maxMembers){
				throw new ProcessException(sprintf(_('Sorry, at the moment max %d member limit apply for private Wikis. The Site would have to be upgraded to allow more members.'), $maxMembers));	
			}
		}
			
		// all should be fine at this point - add to members
		
		$db = Database::connection();
		$db->begin();
		
		$mem = new DB_Member();
		$mem->setDateJoined(new ODate());
		$mem->setSiteId($site->getSiteId());
		$mem->setUserId($user->getUserId());

		$mem->save();
		
		$ml = new DB_MembershipLink();
		$ml->setUserId($user->getUserId());
		$ml->setSiteId($site->getSiteId());
		$ml->setDate(new ODate());
		$ml->setType('EMAIL_INVITATION');
		$ml->setByUserId($inv->getUserId());
		$ml->save();
		
		// add to contacts?
		$sender = DB_OzoneUserPeer::instance()->selectByPrimaryKey($inv->getUserId());
		if($inv->getToContacts() && $sender->getUserId()!=$user->getUserId()){
			try{
				
				// check if contact already exists
				$c = new Criteria();
				$c->add("user_id", $user->getUserId());
				$c->add("target_user_id", $sender->getUserId());
				$con0 = DB_ContactPeer::instance()->selectOne($c);
				if(!$con0){
					$con = new DB_Contact();
					$con->setUserId($user->getUserId());	
					$con->setTargetUserId($sender->getUserId());
					$con->save();
				}
			}catch(Exception $e){}
			
			try{
				// check if contact already exists
				$c = new Criteria();
				$c->add("user_id", $sender->getUserId());
				$c->add("target_user_id", $user->getUserId());
				$con0 = DB_ContactPeer::instance()->selectOne($c);
				if(!$con0){
					$con = new DB_Contact();
					$con->setUserId($sender->getUserId());	
					$con->setTargetUserId($user->getUserId());
					$con->save();
				}
			}catch(Exception $e){}
		}
		
		// set accepted
		$inv->setAccepted(true);
		$inv->save();

		// create a notification
		
		AdminNotificationMaker::instance()->acceptedEmailInvitation($inv, $user);
		
		$db->commit();
		
		$runData->contextAdd("site", $site);
	}
	
}
