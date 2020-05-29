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

class UserInvitationAction extends SmartyAction {
	
	public function perform($runData){}
	
	public function sendEmailInvitationsEvent($runData){
		$pl = $runData->getParameterList();
		$user = $runData->getUser();
		$site = $runData->getTemp("site");
		
		// is user allowed to send invitations?
		$siteSettings = $site->getSettings();
		$sendingEnabled = $siteSettings->getAllowMembersInvite();
		if(!$sendingEnabled){
			throw new ProcessException(_("Users are not allowed to send invitations to this Wiki."));
		}
		
		if(!$user){
			throw new ProcessException(_("You are not logged in."));	
		}
		// check if a member
		$c = new Criteria();
		$c->add("user_id", $user->getUserId());
		$c->add("site_id", $site->getSiteId());
		$mem = DB_MemberPeer::instance()->selectOne($c);
		if(!$mem){
			throw new ProcessException(_("Only members of this Wiki are allowed to send invitations."));	
		}

		$json = new JSONService(SERVICES_JSON_LOOSE_TYPE);
		$addresses = $json->decode($pl->getParameterValue("addresses"));
		
		$message = $pl->getParameterValue("message");
		// check if data is valid
		
		if(count($addresses) > 20){
			throw new ProcessException(_("You should not send more than 20 invitations at once."));	
		}
		
		foreach($addresses as $address){
			$email = trim($address[0]);
			$name = trim($address[1]);
			if(!preg_match("/^[_a-zA-Z0-9-]+(\.[_a-zA-Z0-9-]+)*@[a-zA-Z0-9-]+(\.[a-zA-Z0-9-]+)+$/", $email) || strlen8($email)>70 || strlen($email) == 0){
				throw new ProcessException(sprintf(_('Email "%s" is not valid.'), htmlspecialchars($email)), "bad_email");	
			}
			
			if(preg_match(';://;',$name) || preg_match(';\.www;i', $name) || strlen($name)>50 || strlen($name) == 0){
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
		$user = $runData->getUser();
		
		$invitationId = $pl->getParameterValue("invitationId");
		
		$c = new Criteria();
		$c->add("invitation_id", $invitationId);
		$c->add("site_id", $site->getSiteId());
		
		$inv = DB_EmailInvitationPeer::instance()->selectOne($c);
		
		if(!$inv){
			throw new ProcessException(_("Invitation could not be found."), "no_invitation");	
		}
		if($inv->getUserId() != $user->getUserId()){
			throw new ProcessException(_("This invitation does not seem to be sent by you..."));	
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
		
		if(!res){
			throw new ProcessException("Email to this recipient could not be sent for some reason.");	
		}
		$inv->setAttempts($inv->getAttempts()+1);
		$inv->save();
		$db->commit();
	
	}
	
}
