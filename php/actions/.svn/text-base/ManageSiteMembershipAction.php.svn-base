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

class ManageSiteMembershipAction extends SmartyAction {
	
	public function isAllowed($runData){
		WDPermissionManager::instance()->hasPermission('manage_site', $runData->getUser(), $runData->getTemp("site"));	
		return true;
	}
	
	public function perform($r){}
	
	public function saveMemberPolicyEvent($runData){
		
		$db = Database::connection();;
		$db->begin();
		
		$site = $runData->getTemp("site");
		// get current settings:
		
		$settings = $site->getSettings();
		$superSettings = $site->getSuperSettings();
		
		// get submitted parameters - sorry for the manual way, 
		// we did not want to use ozone mechanisms for form submissions
		
		$pl = $runData->getParameterList();
		$byApply = $pl->getParameterValue("by_apply");
		if($byApply != null){ 
			$byApply = true;
		} else {
			$byApply = false;	
		}

		$byPassword = $pl->getParameterValue("by_password");
		if($byPassword != null){ 
			$byPassword = true;
		} else {
			$byPassword = false;	
		}
		$password = $pl->getParameterValue("password");
		
		///// compare to current version
		
		$changed = false;
		
		if($settings->getAllowMembershipByApply() !== $byApply){
			$settings->setAllowMembershipByApply($byApply);
			$changed = true;	
		}
		
		if($settings->getAllowMembershipByPassword() !== $byPassword){
			$settings->setAllowMembershipByPassword($byPassword);
			$changed = true;	
		}
		if($settings->getMembershipPassword() !== $password){
			$settings->setMembershipPassword($password);
			$changed = true;	
		}

		if($changed){
			$settings->save();	
		}
		$db->commit();	
		if (GlobalProperties::$UI_SLEEP) { sleep(1); }
	}
	
	public function addAdminEvent($runData){
		$db = Database::connection();;
		$db->begin();
		
		$site = $runData->getTemp("site");
		$pl = $runData->getParameterList();
		$userId = $pl->getParameterValue("user_id");
			
		// add administership
		$a = new DB_Admin();
		$a->setUserId($userId);
		$a->setSiteId($site->getSiteId());
		$a->setDateJoined(new ODate());
	
		$runData->ajaxResponseAdd("result", 'added');
		$runData->ajaxResponseAdd("userId", $userId);
		$db->commit();	
		if (GlobalProperties::$UI_SLEEP) { sleep(1); }
	}

	public function inviteMemberEvent($runData){
		$site = $runData->getTemp("site");
		$pl = $runData->getParameterList();
		$userId = $pl->getParameterValue("user_id");
		
		$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($userId);
		if($user == null){
			throw new ProcessException("Error");
		}	
		
		try{
			WDPermissionManager::instance()->hasPermission("become_member", $user, $site);
		}catch(Exception $e){
			throw new ProcessException(_("It seems that this user is on the blacklist"));	
		}
		
		$c = new Criteria();
		$c->add("user_id", $userId);
		$c->add("site_id", $site->getSiteId());
		
		$mem = DB_MemberPeer::instance()->select($c);
		if(count($mem) > 0){
			throw new ProcessException(_("This user already is a member of this site."), "already_member");
		}
		$invs = DB_MemberInvitationPeer::instance()->select($c);
		if(count($invs) > 0){
			throw new ProcessException(_("This user has been already invited to this site."), "already_invited");
		}
		
		// check if user WISHES to receive invitations
		
		$set = DB_UserSettingsPeer::instance()->selectByPrimaryKey($user->getUserId());
		if(!$set->getReceiveInvitations()){
			throw new ProcessException(_("This user does not wish to receive any invitations.", "wishes_not"));
		}
		
		$db = Database::connection();
		$db->begin();
		// ok, create invitation
		$text = substr(trim($pl->getParameterValue("text")),0,300);
		$inv = new DB_MemberInvitation();
		$inv->setUserId($userId);
		$inv->setByUserId($runData->getUserId());
		$inv->setSiteId($site->getSiteId());
		$inv->setDate(new ODate());
		$inv->setBody($text);
		$inv->save();
	
		// and create a notification too...
		
		NotificationMaker::instance()->newMembershipInvitation($inv);

		$db->commit();
		
		$runData->ajaxResponseAdd("result", 'invited');
		$runData->ajaxResponseAdd("userId", $userId);
	}
	
	public function removeMemberEvent($runData){
		$userId = $runData->getParameterList()->getParameterValue("user_id");
		$ban = $runData->getParameterList()->getParameterValue("ban");
		
		$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($userId);
		
		if($user == null){
			throw new ProcessException("Error");
		}	
		
		$site =$runData->getTemp("site");
		$siteId = $site->getSiteId();
		
		// remember: one can NOT remove the last admin nor himself.
		if($userId == $runData->getUserId()){
			throw new ProcessException(_('You can not remove yourself! Use "your account" panel instead.'), "not_yourself");
		}
		
		$db = Database::connection();
		$db->begin();
		
		// check if is admin. can not remove last admin.
		$c = new Criteria();
		$c->add("user_id", $userId);
		$c->add("site_id", $siteId);
		$admin = DB_AdminPeer::instance()->selectOne($c);
		
		if($admin && $admin->getFounder()){
			throw new ProcessException(_("The founder of the site can not be removed."). "founder_nonremovable");	
		}
		
		if($admin){
			$c2 = new Criteria();
			$c2->add("site_id", $siteId);
			$acount = DB_AdminPeer::instance()->selectCount($c2);
			if($acount == 1){ // BUT this meand "yourself"
				throw new ProcessException(_("You can not remove the last admin."), "last_admin)");
			}
		}
		
		$c = new Criteria();
		$c->add("user_id", $userId);
		$c->add("site_id", $siteId);
		
		DB_MemberPeer::instance()->delete($c);
		DB_ModeratorPeer::instance()->delete($c);
		DB_AdminPeer::instance()->delete($c);
		
		NotificationMaker::instance()->removedFromMembers($site, $user);
		
		// ban or not?
		
		if($ban){
			$c = new Criteria();
			$c->add("site_id", $site->getSiteId());
			$c->add("user_id", $userId);
			$bl = DB_UserBlockPeer::instance()->selectOne($c);
			if($bl){
				throw new ProcessException(_("Error occured."));	
			}	
			$block = new DB_UserBlock();
			$block->setSiteId($site->getSiteId());
			$block->setUserId($userId);
			$block->setDateBlocked(new ODate());
			
			$block->save();
		}
		
		$db->commit();

	}
	
	public function toModeratorsEvent($runData){
		$userId = $runData->getParameterList()->getParameterValue("user_id");
		$siteId = $runData->getTemp("site")->getSiteId();
		
		$site =$runData->getTemp("site");
		
		$db = Database::connection();
		$db->begin();
		
		$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($userId);
		if($user == null){
			$runData->ajaxResponseAdd("status", "no_user");
			$runData->ajaxResponseAdd("message", _("The user does not exist? This should not happen."));
			$db->commit();	
			return;
		}
		// check if a member
		$c = new Criteria();
		$c->add("user_id", $userId);
		$c->add("site_id", $siteId);
		$mem = DB_MemberPeer::instance()->selectOne($c);
		if($mem == null){
			$runData->ajaxResponseAdd("status", "not_member");
			$runData->ajaxResponseAdd("message", _("The user is not a member of this site (anymore)."));
			$db->commit();	
			return;	
		}
		
		// check if not already a moderator
		$mod = DB_ModeratorPeer::instance()->selectOne($c);
		if($mod != null){
			$runData->ajaxResponseAdd("status", "already_moderator");
			$runData->ajaxResponseAdd("message", _("The user is already a moderator of this site."));
			$db->commit();	
			return;	
		}
		
		// check if not already an admin. The roles should not duplicate.
		$mod = DB_AdminPeer::instance()->selectOne($c);
		if($mod != null){
			$runData->ajaxResponseAdd("status", "already_admin");
			$runData->ajaxResponseAdd("message", _("The user is already an administrator of this site."));
			$db->commit();	
			return;	
		}
		
		// ok, add now!
		
		$mod = new DB_Moderator();
		$mod->setSiteId($siteId);
		$mod->setUserId($userId);
		$mod->save();
		
		NotificationMaker::instance()->addedToModerators($site, $user);

		$runData->ajaxResponseAdd("userName", $user->getNickName());
		
		$db->commit();
		
	}	
	
	public function removeModeratorEvent($runData){
		$userId = $runData->getParameterList()->getParameterValue("user_id");
		
		$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($userId);
		
		if($user == null){
			throw new ProcessException("Error");
		}

		$siteId = $runData->getTemp("site")->getSiteId();
		$site =$runData->getTemp("site");
		// check if IS a moderator
		
		$db = Database::connection();
		$db->begin();
		
		$c = new Criteria();
		$c->add("user_id", $userId);
		$c->add("site_id", $siteId);
		$mod = DB_ModeratorPeer::instance()->selectOne($c);
		if($mod == null){
			$runData->ajaxResponseAdd("status", "not_already");
			$runData->ajaxResponseAdd("message", _("This user is not a moderator already."));
			$db->commit();
			return;	
		}
		DB_ModeratorPeer::instance()->delete($c);
		
		NotificationMaker::instance()->removedFromModerators($site, $user);

		$db->commit();
	}	
	
	public function toAdminsEvent($runData){
		$userId = $runData->getParameterList()->getParameterValue("user_id");
		$siteId = $runData->getTemp("site")->getSiteId();
		
		$site =$runData->getTemp("site");
		
		$db = Database::connection();
		$db->begin();
		
		$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($userId);
		if($user == null){
			$runData->ajaxResponseAdd("status", "no_user");
			$runData->ajaxResponseAdd("message", _("The user does not exist? This should not happen."));
			$db->commit();	
			return;
		}
		// check if a member
		$c = new Criteria();
		$c->add("user_id", $userId);
		$c->add("site_id", $siteId);
		$mem = DB_MemberPeer::instance()->selectOne($c);
		if($mem == null){
			$runData->ajaxResponseAdd("status", "not_member");
			$runData->ajaxResponseAdd("message", _("The user is not a member of this site (anymore)."));
			$db->commit();	
			return;	
		}
		
		// check if not already a moderator
		$mod = DB_ModeratorPeer::instance()->selectOne($c);
		if($mod != null){
			$runData->ajaxResponseAdd("status", "already_moderator");
			$runData->ajaxResponseAdd("message",_("The user is already a moderator of this site."));
			$db->commit();	
			return;	
		}
		
		// check if not already an admin. The roles should not duplicate.
		$mod = DB_AdminPeer::instance()->selectOne($c);
		if($mod != null){
			$runData->ajaxResponseAdd("status", "already_admin");
			$runData->ajaxResponseAdd("message", _("The user is already an administrator of this site."));
			$db->commit();	
			return;	
		}
		
		WDPermissionManager::instance()->canBecomeAdmin($user);
		
		// ok, add now!
		
		$mod = new DB_Admin();
		$mod->setSiteId($siteId);
		$mod->setUserId($userId);
		$mod->save();
		
		// and create a notification too...
		NotificationMaker::instance()-> addedToAdmins($site, $user);
		
		$runData->ajaxResponseAdd("userName", $user->getNickName());
		
		$db->commit();
		
	}	
	
	public function removeAdminEvent($runData){
		$userId = $runData->getParameterList()->getParameterValue("user_id");
		$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($userId);
		
		if($user == null){
			throw new ProcessException("Error");
		}	
		
		$siteId = $runData->getTemp("site")->getSiteId();
		$site =$runData->getTemp("site");
		// check if IS an admin
		
		$db = Database::connection();
		$db->begin();
		
		$c = new Criteria();
		$c->add("user_id", $userId);
		$c->add("site_id", $siteId);
		$admin = DB_AdminPeer::instance()->selectOne($c);
		
		if($admin && $admin->getFounder()){
			throw new ProcessException(_("The original founder of the site can not be removed."), "founder_nonremovable");	
		}
		
		if($admin == null){
			$runData->ajaxResponseAdd("status", "not_already");
			$runData->ajaxResponseAdd("message", _("This user is not an administator any more."));
			$db->commit();
			return;	
		}
		
		if($userId === $runData->getUserId()){
			$runData->ajaxResponseAdd("status", "not_yourself");
			$runData->ajaxResponseAdd("message", _("You can not remove yourself from site admins."));
			$db->commit();
			return;	
		}
		
		$c2 = new Criteria();
		$c2->add("site_id", $siteId);
		$acount = DB_AdminPeer::instance()->selectCount($c2);
		if($acount == 1){ // BUT this meand "yourself"
			$runData->ajaxResponseAdd("status", "last_admin");
			$runData->ajaxResponseAdd("message", _("You can not remove the last admin."));
			$db->commit();
			return;
		}
		
		DB_AdminPeer::instance()->delete($c);
		
		// and create a notification too...
		NotificationMaker::instance()-> removedFromAdmins($site, $user);
		
		$db->commit();
	}	

	public function acceptApplicationEvent($runData){
		$pl = $runData->getParameterList();
	
		$userId = $pl->getParameterValue("user_id");
		$site = $runData->getTemp("site");
		$siteId = $site->getSiteId();
		
		$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($userId);
		
		if($user == null){
			throw new ProcessException("Error");
		}	
		
		$type = $pl->getParameterValue("type");
		$text = $pl->getParameterValue("text");
		
		if($type !== 'accept' && $type !== 'decline'){
			throw new ProcessException("Invalid action", "invalid_action");	
		}
		
		$c = new Criteria();
		$c->add("user_id", $userId);
		$c->add("site_id", $siteId);
		
		$db = Database::connection();
		$db->begin();
		
		$application = DB_MemberApplicationPeer::instance()->selectOne($c);
		if($application == null){
			throw new ProcessException(_("This application does not exist (anymore)."), "no_application");
		}
		
		if($type=="accept"){
			// add to members
			$mem = new DB_Member();
			$mem->setUserId($userId);
			$mem->setSiteId($siteId);
			$mem->setDateJoined(new ODate());
			$mem->save();	
			
			$ml = new DB_MembershipLink();
			$ml->setUserId($userId);
			$ml->setSiteId($site->getSiteId());
			$ml->setDate(new ODate());
			$ml->setType('APPLICATION_ACCEPTED');
			$ml->setByUserId($runData->getUser()->getUserId());
			$ml->save();
			
			NotificationMaker::instance()->membershipApplicationAccepted($site, $user);
		}else{
			NotificationMaker::instance()->membershipApplicationDeclined($site, $user);
		}
		
		$application->setReply($text);
		if($type == "accept"){
			$application->setStatus("accepted");
		}else{
			$application->setStatus("declined");
		}
		$application->save();
		
		$db->commit();
	}
	
	public function saveModeratorPermissionsEvent($runData){
		$pl = $runData->getParameterList();
		$moderatorId = $pl->getParameterValue("moderatorId");
		
		if($moderatorId == null || !is_numeric($moderatorId)){
			throw new ProcessException(_("Moderator does not exist."));	
		}
		$mod = DB_ModeratorPeer::instance()->selectByPrimaryKey($moderatorId);
		
		if($mod == null || $mod->getSiteId() != $runData->getTemp("site")->getSiteId()){
			throw new ProcessException(_("Moderator does not exist."));	
		}	
		
		$ps = '';
		if($pl->getParameterValue("pages")){
			$ps.='p';	
		}
		if($pl->getParameterValue("forum")){
			$ps.='f';	
		}
		if($pl->getParameterValue("users")){
			$ps.='u';	
		}
		
		$mod->setPermissions($ps);
		$mod->save();
		
		if (GlobalProperties::$UI_SLEEP) { sleep(1); }
	}
	
	public function sendEmailInvitationsEvent($runData){
		$pl = $runData->getParameterList();
		$user = $runData->getUser();
		$site = $runData->getTemp("site");
		
		$json = new JSONService(SERVICES_JSON_LOOSE_TYPE);
		$addresses = $json->decode($pl->getParameterValue("addresses"));
		
		$message = $pl->getParameterValue("message");
		// check if data is valid
		
		if(count($addresses) > 200){
			throw new ProcessException(_("You should not send more than 200 invitations at once."));	
		}
		
		foreach($addresses as $address){
			$email = trim($address[0]);
			$name = trim($address[1]);
			if(!preg_match("/^[_a-zA-Z0-9-]+(\.[_a-zA-Z0-9-]+)*@[a-zA-Z0-9-]+(\.[a-zA-Z0-9-]+)+$/", $email) || strlen($email)>70 || strlen($email) == 0){
				throw new ProcessException(sprintf(_('Email "%s" is not valid.'), htmlspecialchars($email)), "bad_email");	
			}
			
			if(preg_match(';://;',$name) || preg_match(';\.www;i', $name) || strlen8($name)>50 || strlen8($name) == 0){
				throw new ProcessException(sprintf(_('Recipient\'s name "%s" is not valid.'), htmlspecialchars($name)), "bad_name");
			}
			
			//check if "email" is not already a member of this site...
			$q = " SELECT * FROM member, ozone_user WHERE member.site_id='".$site->getSiteId()."' AND ozone_user.name='".db_escape_string($email)."' AND member.user_id = ozone_user.user_id LIMIT 1";
			$c = new Criteria();
			$c->setExplicitQuery($q);
			$m = DB_MemberPeer::instance()->selectOne($c);
			if($m){
				throw new ProcessException(sprintf(_('User with the email address "%s" is already a member of this Site. Remove him from the list and send invitations again.'), htmlspecialchars($email)), 'aleady_member');		
			}
			
			// check if not sent already to this address.
			$c = new Criteria();
			$c->add("email", $email);
			$c->add("site_id", $site->getSiteId());
			$ii = DB_EmailInvitationPeer::instance()->selectOne($c);
			
			if($ii){
				throw new ProcessException(sprintf(_('User with the email address "%s" has been already invited to this Site. Remove him from the list and send invitations again. If you want to resend an invitation please rather look at the history of sent invitations.'), htmlspecialchars($email)), 'aleady_member');					
			}
		}
		
		if(preg_match(';://;',$message) || preg_match(';www\.;i', $message) ){
			throw new ProcessException(_('The message should not contain any links to websites.'), "bad_message");
		}
		if($message != "" && strlen($message)>1000){
			throw new ProcessException(_('The message seems to be too long. Max 1000 characters are allowed.'), "bad_message");
		}
		
		// now prepare invitation and send!
		
		$db = Database::connection();

		foreach($addresses as $address){
			$email = trim($address[0]);
			$name = trim($address[1]);
			$db->begin(); // each invitation makes a separate transaction
			
			$hash = substr(md5($name.$email).time(),0,20);
			
			$inv = new DB_EmailInvitation();
			$inv->setHash($hash);
			$inv->setEmail($email);
			$inv->setName($name);
			$inv->setUserId($user->getUserId());
			$inv->setSiteId($site->getSiteId());
			$inv->setMessage($message);
			$inv->setDate(new ODate());
			if($address[2]){
				$inv->setToContacts(true);	
			}
			
			// prepare and send email
			$profile = $user->getProfile();
			
			$oe = new OzoneEmail();
			$oe->addAddress($email);
			$oe->setSubject(sprintf(_("[%s] %s invites you to join!"), GlobalProperties::$SERVICE_NAME, $user->getNickName()));
			$oe->contextAdd('user', $user);
			$oe->contextAdd('profile', $profile);
			$oe->contextAdd('hash', $hash);
			$oe->contextAdd("site", $site);
			$oe->contextAdd("message", $message);
			$oe->contextAdd('name', $name);
			
			$oe->setBodyTemplate('MembershipEmailInvitation');
			
			if(!$oe->Send()){
				$inv->setDelivered(false);	
			}else{
				$inv->setDelivered(true);
			}
			
			$inv->save();
		
			$db->commit();
		}
	}
	
	public function deleteEmailInvitationEvent($runData){
		$pl = $runData->getParameterList();
		$site = $runData->getTemp("site");
		
		$invitationId = $pl->getParameterValue("invitationId");
		
		$c = new Criteria();
		$c->add("invitation_id", $invitationId);
		$c->add("site_id", $site->getSiteId());
		
		$inv = DB_EmailInvitationPeer::instance()->selectOne($c);
		
		if(!$inv){
			throw new ProcessException(_("Invitation could not be found."), "no_invitation");	
		}
		
		// delete now
		DB_EmailInvitationPeer::instance()->deleteByPrimaryKey($invitationId);
	}
	
	public function resendEmailInvitationEvent($runData){
		$pl = $runData->getParameterList();
		$site = $runData->getTemp("site");	
		
		$invitationId = $pl->getParameterValue("invitationId");
		
		$message2 = trim($pl->getParameterValue("message"));
		
		$c = new Criteria();
		$c->add("invitation_id", $invitationId);
		$c->add("site_id", $site->getSiteId());
		
		$inv = DB_EmailInvitationPeer::instance()->selectOne($c);
		
		if(!$inv){
			throw new ProcessException(_("Invitation could not be found."), "no_invitation");	
		}
		
		if($inv->getAttempts()>=3){
			throw new ProcessException(_("You can not send more than 3 copies of the invitation."));
		}
		
		if($message2 == ""){
			throw new ProcessException(_('Message should not be empty'));
		}
		
		if(preg_match(';://;',$message2) || preg_match(';\.www;i', $message2) ){
			throw new ProcessException(_('The message should not contain any links to websites.'), "bad_message");
		}
		if($message2 != "" && strlen($message2)>1000){
			throw new ProcessException(_('The message seems to be too long. Max 1000 characters are allowed.'), "bad_message");
		}
		
		$db = Database::connection();
		$db->begin();
		
		// prepare and send email
		$user = $runData->getUser();
		$profile = $user->getProfile();
			
		$oe = new OzoneEmail();
		$oe->addAddress($inv->getEmail());
		$oe->setSubject(sprintf(_("[%s] %s invites you to join! (reminder)"), GlobalProperties::$SERVICE_NAME, $user->getNickName()));
		$oe->contextAdd('user', $user);
		$oe->contextAdd('profile', $profile);
		$oe->contextAdd('hash', $inv->getHash());
		$oe->contextAdd("site", $site);
		$oe->contextAdd("message", $inv->getMessage());
		$oe->contextAdd("message2", $message2);
		$oe->contextAdd('name', $inv->getName());
			
		$oe->setBodyTemplate('MembershipEmailInvitation');
		
		$res = $oe->send();
		
		if(!$res){
			throw new ProcessException("Email to this recipient could not be sent for some reason.");	
		}
		$inv->setAttempts($inv->getAttempts()+1);
		$inv->save();
		$db->commit();
	}
	
	public function letUsersInviteSaveEvent($runData){
		$settings = $runData->getTemp("site")->getSettings();
		$pl = $runData->getParameterList();
		$enable = $pl->getParameterValue("enableLetUsersInvite") == "true";
		$settings->setAllowMembersInvite($enable);
		$settings->save();
		if (GlobalProperties::$UI_SLEEP) { sleep(1); }
		
	}
	
	public function uploadContactsForInvitationsEvent($runData){
		$status = "ok"; // status variable that will be passed to template

		$pl = $runData->getParameterList();
		$file = $_FILES['contactfile'];
		if($file['size'] == 0){
			$status = "zero_size";	
			$runData->contextAdd("status", $status);
			return;		
		}
		
		if($file['error'] !=0){
			$status = "other error";	
			$runData->contextAdd("status", $file['error']);
			return;		
		}
		
		if(!is_uploaded_file($file['tmp_name'])){
			$status = "invalid_file";	
			$runData->contextAdd("status", $status);
			return;		
		}
		
		// read the file, convert encoding...?
		
		$cont = file_get_contents($file['tmp_name']);
		
		$enc = mb_detect_encoding($cont, "UTF-8, UTF-16BE, UTF-16LE, UCS-2, UCS-2BE, UCS-2LE, UTF-16, ASCII");
		if(!$enc){
			$enc = $this->getUnicode($cont);
		}

		if($enc != "UTF-8"){
			$cont = mb_convert_encoding($cont, "UTF-8", $enc);
		}

		//save to a tmp file
		
		$tmpfile = tmpfile();
		fwrite($tmpfile, $cont);
		fseek($tmpfile, 0);
		
		// access as a CSV 	
		$header = fgetcsv($tmpfile);
		// look for name and email
		
		$namePos = 0;
		for($i=0; $i<count($header); $i++){
			if(preg_match(";name;i", $header[$i])){
				$namePos = $i;
				break;
			}	
		}
		$emailPos = 0;
		for($i=0; $i<count($header); $i++){
			if(preg_match(";e\-?mail;i", $header[$i])){
				$emailPos = $i;
				break;
			}	
		}
		
		// read all the rows and get name + email
		$adrs = array();
		while(($data = fgetcsv($tmpfile)) !== false) {
			$name = $data[$namePos];
			$email = $data[$emailPos];
			$adrs[] = array('name' => $name, 'email'=> $email);
		}

		fclose($tmpfile);
		
		//encode adresses
		$json = new JSONService();
		$adrs = $json->encode($adrs);

		$runData->contextAdd("status", $status);
		$runData->contextAdd("adrs", $adrs);
		
	}

	private function getUnicode($string) {
		// thx to the iaddressbook project
	    if(substr($string, 0, 4) == "\0\0\xFE\xFF") return 'UTF-32BE';    //Big Endian
	    if(substr($string, 0, 4) == "\xFF\xFE\0\0") return 'UTF-32LE';    //Little Endian
	    if(substr($string, 0, 2) == "\xFE\xFF") return 'UTF-16BE';        //Big Endian
	    if(substr($string, 0, 2) == "\xFF\xFE") return 'UTF-16LE';        //Little Endian
	    if(substr($string, 0, 3) == "\xEF\xBB\xBF") return 'UTF-8';
	
	    // no match, check for utf-8
	    if($this->isUtf8(substr($string, 0, 512))) return 'UTF-8';
	
	    // heuristics
	    if($string[0] == "\0" && $string[1] == "\0" && $string[2] == "\0" && $string[3] != "\0") return 'UTF-32BE';
	    if($string[0] != "\0" && $string[1] == "\0" && $string[2] == "\0" && $string[3] == "\0") return 'UTF-32LE';
	    if($string[0] == "\0" && $string[1] != "\0" && $string[2] == "\0" && $string[3] != "\0") return 'UTF-16BE';
	    if($string[0] != "\0" && $string[1] == "\0" && $string[2] != "\0" && $string[3] == "\0") return 'UTF-16LE';
	
	    return false;
	}
	
	private function isUtf8($string) {

   		// From http://w3.org/International/questions/qa-forms-utf-8.html
   		return preg_match('%^(?:
	         [\x09\x0A\x0D\x20-\x7E]            # ASCII
	       | [\xC2-\xDF][\x80-\xBF]            # non-overlong 2-byte
	       |  \xE0[\xA0-\xBF][\x80-\xBF]        # excluding overlongs
	       | [\xE1-\xEC\xEE\xEF][\x80-\xBF]{2}  # straight 3-byte
	       |  \xED[\x80-\x9F][\x80-\xBF]        # excluding surrogates
	       |  \xF0[\x90-\xBF][\x80-\xBF]{2}    # planes 1-3
	       | [\xF1-\xF3][\x80-\xBF]{3}          # planes 4-15
	       |  \xF4[\x80-\x8F][\x80-\xBF]{2}    # plane 16
	   )*$%xs', $string);

}
	
}
