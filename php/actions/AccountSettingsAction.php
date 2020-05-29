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

class AccountSettingsAction extends SmartyAction {
	
	public function isAllowed($runData){
		$userId = $runData->getUserId();
		if($userId == null || $userId <1){
			throw new WDPermissionException(_("Not allowed. You should login first."));
		}
		return true;
	}
	
	public function perform($r){}
	
	public function changePasswordEvent($runData){
		$pl = $runData->getParameterList();
		$user = $runData->getUser();
		
		$oldPassword = $pl->getParameterValue("old_password");
		$newPassword1 = ($pl->getParameterValue("new_password1"));
		$newPassword2 = ($pl->getParameterValue("new_password2"));
		
		$oldPassword = trim(CryptUtils::rsaDecrypt($oldPassword));
		$newPassword1 = trim(CryptUtils::rsaDecrypt($newPassword1));
		$newPassword2 = trim(CryptUtils::rsaDecrypt($newPassword2));
		
		$oldPassword = preg_replace("/^__/", '', $oldPassword);
		$newPassword1 = preg_replace("/^__/", '', $newPassword1);
		$newPassword2 = preg_replace("/^__/", '', $newPassword2);
		
		if(md5($oldPassword) !== $user->getPassword()){
			throw new ProcessException(_("Can not change your password. The current password is invalid."), "form_error");
		}
		if($newPassword1 !== $newPassword2){
			throw new ProcessException(_("Can not change your password. New passwords differ but should be identical to eliminate typos."), "form_error");
			
		}
		if(strlen8($newPassword1)<6){
			throw new ProcessException(_("Can not change your password. The new password is too short. Min 6 characters please!"), "form_error");
			
		}
		if(strlen8($newPassword1)>20){
			throw new ProcessException(_("Can not change your password. The new password is too long. Max 20 characters please!"), "form_error");
			
		}
		
		// ok, change the password!!!
		$user->setPassword(md5($newPassword1));
		$user->save();
		
	}
	
	public function changeEmail1Event($runData){
		$pl = $runData->getParameterList();
		
		$email = $pl->getParameterValue("email", "AMODULE");
		
		if($email == null || $email == ''){
			throw new ProcessException(_("Email must be provided."), "no_email");	
		}
		
		if(preg_match("/^[_a-zA-Z0-9-]+(\.[_a-zA-Z0-9-]+)*@[a-zA-Z0-9-]+(\.[a-zA-Z0-9-]+)+$/", $email) ==0){
			throw new ProcessException(_("Valid email must be provided."), "no_email");	
		}	
		
		// check for users with the email
		$c = new Criteria();
		$c->add("email", $email);
		$user = DB_OzoneUserPeer::instance()->selectOne($c);
		
		if($user !== null){
			throw new ProcessException(_("An user with this email already exists. Emails must be unique."), "form_error");		
		}
		
		// generate code
		srand((double)microtime()*1000000);
		$string = md5(rand(0,9999));
		$evcode = substr($string, 2, 6);

		//send a confirmation email to the user.
		$oe = new OzoneEmail();
		$oe->addAddress($email);
		$oe->setSubject(sprintf(_("%s - email address change"), GlobalProperties::$SERVICE_NAME));
		$oe->contextAdd("user", $runData->getUser());
		$oe->contextAdd("email", $email);
		$oe->contextAdd('evcode', $evcode);
		
		$oe->setBodyTemplate('ChangeEmailVerification');
	
		if (!$oe->Send()) {
			throw new ProcessException(_("The email can not be sent to this address."), "form_error");
		} 		 
		
		$runData->sessionAdd("chevcode", $evcode);
		$runData->sessionAdd("ch-nemail", $email);
		$runData->contextAdd("email", $email);	
	}
	
	public function changeEmail2Event($runData){
		$pl = $runData->getParameterList();
		
		$evercode = $pl->getParameterValue("evercode");
		
		if($evercode != $runData->sessionGet("chevcode")){
			throw new ProcessException(_("The verification codes do not match."), "form_error");
		}
		$email = $runData->sessionGet("ch-nemail");
		$runData->sessionDel("ch-nemail");
		$runData->sessionDel("chevcode");
		
		$user = $runData->getUser();
		$user->setName($email);
		$user->setEmail($email);
		$user->save();
		
		$runData->contextAdd("email", $email);
		
	}
	
	public function saveReceiveInvitationsEvent($runData){
		
		$pl = $runData->getParameterList();	
		$receive = $pl->getParameterValue("receive");
		if($receive){
			$receive = true;
		}else{
			$receive = false;
		}
		$us = DB_UserSettingsPeer::instance()->selectByPrimaryKey($runData->getUserId());
		$us->setReceiveInvitations($receive);
		$us->save();
		if (GlobalProperties::$UI_SLEEP) { sleep(1); }
	}
	
	public function saveReceiveMessagesEvent($runData){
		
		$pl = $runData->getParameterList();	
		$from = $pl->getParameterValue("from");
		
		if($from !== "a" && $from !== "mf" && $from !=="f" && $from !== "n"){
			$from = "a";	
		}

		$us = DB_UserSettingsPeer::instance()->selectByPrimaryKey($runData->getUserId());
		$us->setReceivePm($from);
		$us->save();
		if (GlobalProperties::$UI_SLEEP) { sleep(1); }
	}
	
	public function blockUserEvent($runData){
		$pl = $runData->getParameterList();	
		$userId = $pl->getParameterValue("userId");
		
		if($userId == null || !is_numeric($userId)){
			throw new ProcessException(_("Invalid user."), "no_user");
		}	
		
		$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($userId);
		if($user == null){
			throw new ProcessException(_("Invalid user."), "no_user");	
		}		
		
		// check if already blocked
		$c = new Criteria();
		$c->add("user_id", $runData->getUserId());
		$c->add("blocked_user_id", $userId);
		$b = DB_PrivateUserBlockPeer::instance()->selectOne($c);
		if($b){
			throw new ProcessException(_("You already block this user."));
		}
		
		// check max
		$c = new Criteria();
		$c->add("user_id", $runData->getUserId());
		$blockCount = DB_PrivateUserBlockPeer::instance()->selectCount($c);
		
		$maxBlocks = 30;
		
		if($blockCount>$maxBlocks){
			throw new ProcessException("Sorry, you can only block $maxBlocks users max.", "max_block");
		}

		if($userId == $runData->getUserId()){
			throw new ProcessException(_("What is the point in blocking yourself? ;-)"), "not_self");		
		}
			
		$block = new DB_PrivateUserBlock();
		$block->setUserId($runData->getUserId());
		$block->setBlockedUserId($userId);
		$block->save();
			
	}
	
	public function deleteBlockEvent($runData){
		$pl = $runData->getParameterList();	
		$blockedUserId = $pl->getParameterValue("userId");
		$userId = $runData->getUserId();
		
		$c = new Criteria();
		$c->add("user_id", $userId);
		$c->add("blocked_user_id", $blockedUserId);
		
		DB_PrivateUserBlockPeer::instance()->delete($c);

	}
	
	public function saveReceiveDigestEvent($runData){
		$pl = $runData->getParameterList();
		$user = $runData->getUser();
		
		$receive = (bool) $pl->getParameterValue("receive");

		$settings = $user->getSettings();
		if($receive != $settings->getReceiveDigest()){
			$settings->setReceiveDigest($receive);
			$settings->save();	
		}	
		
		if (GlobalProperties::$UI_SLEEP) { sleep(1); }
	}
	
	public function saveReceiveNewsletterEvent($runData){
		$pl = $runData->getParameterList();
		$user = $runData->getUser();
		
		$receive = (bool) $pl->getParameterValue("receive");

		$settings = $user->getSettings();
		if($receive != $settings->getReceiveNewsletter()){
			$settings->setReceiveNewsletter($receive);
			$settings->save();	
		}	
		
		if (GlobalProperties::$UI_SLEEP) { sleep(1); }
	}
	
	public function saveLanguageEvent($runData){
		$pl = $runData->getParameterList();
		$user = $runData->getUser();
		
		$lang = $pl->getParameterValue("language");
		
		if($lang !== "pl" && $lang !=="en"){
			throw new ProcessException(_("Error selecting the language"));	
		}
		
		$user->setLanguage($lang);
		$user->save();
		
		$runData->ajaxResponseAdd("language", $lang);
	}
	
}
