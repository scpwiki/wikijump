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

class AdminNotificationMaker {
	
	private static $instance;
	
	public static function instance(){
		if(self::$instance == null){
			self::$instance = new AdminNotificationMaker();
		}
		return self::$instance;	
	}
	
	public function newMemberApplication($application){
		$user = $application->getUser();
		
		$siteId = $application->getSiteId();
		$site = DB_SitePeer::instance()->selectByPrimaryKey($siteId);
		
		$not = new DB_AdminNotification();
		$not->setSiteId($site->getSiteId());
		
		$not->setType("NEW_MEMBER_APPLICATION");

		$not->setDate(new ODate());
		
		$extra = array();
		$extra['application_id'] = $application->getApplicationId();
		$extra['from_user_id'] = $user->getUserId();
		//$extra['urls'] = array(	array('check pending applications','http://'.$site->getDomain().'/admin:manage/start/ma')
	
		/*
		 * format for urls is:
		 * 0 - anchor
		 * 1 - href
		 * 2 - onclick
		 */
		 
		$not->setExtra($extra);
		
		$not->save();	
	}
	
	public function newMemberByPassword($site, $user){
		
		$not = new DB_AdminNotification();
		$not->setSiteId($site->getSiteId());
		
		$not->setType("NEW_MEMBER_BY_PASSWORD");

		$not->setDate(new ODate());
		
		$extra = array();
		$extra['user_id'] = $user->getUserId();
		//$extra['urls'] = array(	array('site members','http://'.$site->getDomain().'/admin:manage/start/members-list')
		$not->setExtra($extra);
		
		$not->save();	
	}
	
	public function acceptedEmailInvitation($inv, $user){
		
		$site = DB_SitePeer::instance()->selectByPrimaryKey($inv->getSiteId());
		
		$not = new DB_AdminNotification();
		$not->setSiteId($site->getSiteId());
		
		$not->setType("NEW_MEMBER_BY_EMAIL_INVITATION");

		$not->setDate(new ODate());
		
		$extra = array();
		$extra['user_id'] = $user->getUserId();
		//$extra['urls'] = array(	array('site members','http://'.$site->getDomain().'/admin:manage/start/members-list')
		$not->setExtra($extra);
		
		$not->save();	
	}
	
	public function memberResigned($site, $user){
		
		$not = new DB_AdminNotification();
		$not->setSiteId($site->getSiteId());
		
		$not->setType("MEMBER_RESIGNED");

		$not->setDate(new ODate());
		
		$extra = array();
		$extra['user_id'] = $user->getUserId();
		//$extra['urls'] = array(	array('site members','http://'.$site->getDomain().'/admin:manage/start/members-list')
		$not->setExtra($extra);
		
		$not->save();	
	}
	
	public function moderatorResigned($site, $user){
		
		$not = new DB_AdminNotification();
		$not->setSiteId($site->getSiteId());
		
		$not->setType("MODERATOR_RESIGNED");

		$not->setDate(new ODate());
		
		$extra = array();
		$extra['user_id'] = $user->getUserId();
		//$extra['urls'] = array(	array('site moderators','http://'.$site->getDomain().'/admin:manage/start/moderators'),
		//	array('site members','http://'.$site->getDomain().'/admin:manage/start/members-list')
		$not->setExtra($extra);
		
		$not->save();	
	}
	
	public function adminResigned($site, $user){
		
		$not = new DB_AdminNotification();
		$not->setSiteId($site->getSiteId());
		
		$not->setType("ADMIN_RESIGNED");

		$not->setDate(new ODate());
		
		$extra = array();
		//$extra['urls'] = array(	array('site adminitrators','http://'.$site->getDomain().'/admin:manage/start/admins'),
		//	array('site members','http://'.$site->getDomain().'/admin:manage/start/members-list')
		$extra['user_id'] = $user->getUserId();	
		$not->setExtra($extra);
		
		$not->save();	
	}
	
	public function memberInvitationAccepted($site, $user){
		$not = new DB_AdminNotification();
		$not->setSiteId($site->getSiteId());
		
		$not->setType("INVITATION_ACCEPTED");

		$not->setDate(new ODate());
		
		$extra = array();
		//$extra['urls'] = array(	array('site members','http://'.$site->getDomain().'/admin:manage/start/members-list')
		$extra['user_id'] = $user->getUserId();	
		$not->setExtra($extra);
		
		$not->save();	
	}
	public function memberInvitationDeclined($site, $user){
		$not = new DB_AdminNotification();
		$not->setSiteId($site->getSiteId());
		
		$not->setType("INVITATION_DECLINED");

		$not->setDate(new ODate());
		
		$extra = array();
		$extra['user_id'] = $user->getUserId();	
		//$extra['urls'] = array(	array('site members','http://'.$site->getDomain().'/admin:manage/start/members-list')
		$not->setExtra($extra);
		
		$not->save();	
	}

}
