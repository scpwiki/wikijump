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

class NotificationMaker {
	
	private static $instance;
	
	public static function instance(){
		if(self::$instance == null){
			self::$instance = new NotificationMaker();
		}
		return self::$instance;	
	}
	
	public function privateMessageNotification($message){
		$fromUser = $message->getFromUser();
		
		$not = new DB_Notification();
		$not->setUserId($message->getToUserId());
		$not->setType("new_private_message");
		
		$body = 'You have a new private message in your <a href="http://' . GlobalProperties::$URL_HOST . '/account:you/start/messages">Inbox</a>!<br/>';
		$body .= "From: ".WDRenderUtils::renderUser($fromUser)."<br/>";
		$body .= 'Subject: <a href="http://' . GlobalProperties::$URL_HOST . '/account:you/start/messages/inboxmessage/'.$message->getMessageId().'">'.htmlspecialchars($message->getSubject()).'</a><br/>';
		$body .= 'Preview (first few words): ';
		$body .= $message->getPreview();
		
		$not->setDate(new ODate());
		
		$extra = array();
		$extra['message_id'] = $message->getMessageId();
		$extra['from_user_id'] = $message->getFromUserId();
		$extra['subject'] = $message->getSubject();
		$extra['preview'] = $message->getPreview();
		
		//$extra['urls'] = array(	array('read the message','http://www.wikidot.com/account:you/start/messages/inboxmessage/'.$message->getMessageId()),
	
		/*
		 * format for urls is:
		 * 0 - anchor
		 * 1 - href
		 * 2 - onclick
		 */
		 
		$not->setExtra($extra);
		
		$not->save();	
	}
	
	public function newMembershipInvitation($invitation){
		$site = DB_SitePeer::instance()->selectByPrimaryKey($invitation->getSiteId());
		$not = new DB_Notification();
		$not->setUserId($invitation->getUserId());
		$not->setType("new_membership_invitation");
		
		$extra = array();
		$extra['site_id'] = $site->getSiteId();
		$extra['site_name'] = $site->getName();
		$extra['site_domain'] = $site->getDomain();
		$not->setExtra($extra);
		$not->setDate(new ODate());
		$not->save();	
	}
	
	public function removedFromMembers($site, $user){
		// and create a notification too...
		$not = new DB_Notification();
		$not->setUserId($user->getUserId());
		$not->setType("removed_from_members");

		$extra = array();
		$extra['site_id'] = $site->getSiteId();
		$extra['site_name'] = $site->getName();
		$extra['site_domain'] = $site->getDomain();
		
		$not->setExtra($extra);
		$not->setDate(new ODate());
		$not->save();
	}
	
	public function addedToModerators($site, $user){
		// and create a notification too...
		$not = new DB_Notification();
		$not->setUserId($user->getUserId());
		$not->setType("added_to_moderators");

		$extra['site_id'] = $site->getSiteId();
		$extra['site_name'] = $site->getName();
		$extra['site_domain'] = $site->getDomain();
		
		$not->setExtra($extra);
		
		$not->setDate(new ODate());
		$not->save();
	}
	
	public function removedFromModerators($site, $user){
		// and create a notification too...
		$not = new DB_Notification();
		$not->setUserId($user->getUserId());
		$not->setType("removed_from_moderators");

		$extra['site_id'] = $site->getSiteId();
		$extra['site_name'] = $site->getName();
		$extra['site_domain'] = $site->getDomain();
		$not->setExtra($extra);
		
		$not->setDate(new ODate());
		$not->save();
	}

	public function addedToAdmins($site, $user){
		// and create a notification too...
		$not = new DB_Notification();
		$not->setUserId($user->getUserId());
		$not->setType("added_to_administrators");

		$extra['site_id'] = $site->getSiteId();
		$extra['site_name'] = $site->getName();
		$extra['site_domain'] = $site->getDomain();
		$not->setExtra($extra);
		
		$not->setDate(new ODate());
		$not->save();
	}
	
	public function removedFromAdmins($site, $user){
		// and create a notification too...
		$not = new DB_Notification();
		$not->setUserId($user->getUserId());
		$not->setType("removed_from_administrators");

		$extra['site_id'] = $site->getSiteId();
		$extra['site_name'] = $site->getName();
		$extra['site_domain'] = $site->getDomain();
		$not->setExtra($extra);
		
		$not->setDate(new ODate());
		$not->save();
	}
	
	public function membershipApplicationAccepted($site, $user){
		$not = new DB_Notification();
		$not->setUserId($user->getUserId());
		$not->setType("membership_application_accepted");	
		
		//$body = 'Your membership application to the site <a href="http://'.$site->getDomain().'">"'.htmlspecialchars($site->getName()).'"</a> has been accepted. ' .
		//$urls = array(	array('your applications', "http://www.wikidot.com/account:you/start/applications"),
		$extra['site_id'] = $site->getSiteId();
		$extra['site_name'] = $site->getName();
		$extra['site_domain'] = $site->getDomain();
		$not->setExtra($extra);
		
		$not->setDate(new ODate());
		$not->save();		
	}
	public function membershipApplicationDeclined($site, $user){
		$not = new DB_Notification();
		$not->setUserId($user->getUserId());
		$not->setType("membership_application_declined");	
		
		//$urls = array(	array('your applications', "http://www.wikidot.com/account:you/start/applications"),
		$extra['site_id'] = $site->getSiteId();
		$extra['site_name'] = $site->getName();
		$extra['site_domain'] = $site->getDomain();
		$not->setExtra($extra);
		
		$not->setDate(new ODate());
		$not->save();		
	}
}
