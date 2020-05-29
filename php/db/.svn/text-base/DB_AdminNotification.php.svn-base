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
 * @package Wikidot_Db
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

/**
 * Object Model mapped class.
 *
 */
class DB_AdminNotification extends DB_AdminNotificationBase {

		/**
	 * Generates notification title based on the type
	 */
	public function getTitle(){
		$type = $this->getType();
		switch($type){
			case 'NEW_MEMBER_APPLICATION':
				$title = _("New member application");
				break;	
			case 'INVITATION_ACCEPTED':
				$title = _("Membership invitation accepted");
				break;	
			case 'INVITATION_DECLINED':
				$title = _("Membership invitation declined");
				break;	
			case 'NEW_MEMBER_BY_PASSWORD':
				$title = _("New member joined");
				break;	
			case 'MEMBER_RESIGNED':
				$title = _("A member has left");
				break;	
			case 'MODERATOR_RESIGNED':
				$title = _("A moderator resigned");
				break;	
			case 'ADMIN_RESIGNED':
				$title = _("An administrator resigned");
				break;	
			case 'NEW_MEMBER_BY_EMAIL_INVITATION':
				$title = _("Email invitation accepted");
		}
		
		return $title;
	}
	
	public function setExtra($data){
		parent::setExtra(serialize($data));	
	}
	
	public function getExtra(){
		return unserialize(pg_unescape_bytea(parent::getExtra()));	
	}
	
	public function save(){
		$key = "adminnotificationsfeed..".$this->getSiteId();
		$mc = OZONE::$memcache;	
		$mc->delete($key);
		return parent::save();
	}
	
	public function getBody(){
		
		if(parent::getBody() != ""){
			return parent::getBody();
		}
		$type = $this->getType();
		$extra = $this->getExtra();
		$lang = OZONE::getRunData()->getLanguage();
		switch($type){
			case 'NEW_MEMBER_APPLICATION':
				$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($extra['from_user_id']);
				$body = sprintf(_('There is a new member application from user %s.'), WDRenderUtils::renderUser($user));
				break;	
			case 'INVITATION_ACCEPTED':
				$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($extra['user_id']);
				$body = sprintf(_('The user %s has accepted the invitation and is now a member of the site.'), WDRenderUtils::renderUser($user));
				break;	
			case 'INVITATION_DECLINED':
				$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($extra['user_id']);
				$body = sprintf(_('The user %s has not accepted the invitation.'), WDRenderUtils::renderUser($user));
				break;	
			case 'NEW_MEMBER_BY_PASSWORD':
				$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($extra['user_id']);
				$body = sprintf(_('A new member joined the site: %s - by providing a valid membership password.'), WDRenderUtils::renderUser($user));
				break;	
			case 'NEW_MEMBER_BY_EMAIL_INVITATION':
				$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($extra['user_id']);
				$body = sprintf(_('A new user (%s) accepted the invitation and is now a member of the Site.'), WDRenderUtils::renderUser($user));
				break;	
			case 'MEMBER_RESIGNED':
				$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($extra['user_id']);
				$body = sprintf(_('The user %s is no longer a site member. Resigned.'), WDRenderUtils::renderUser($user));
				break;	
			case 'MODERATOR_RESIGNED':
				$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($extra['user_id']);
				$body = sprintf(_('The user %s resigned from being a moderator of this site.'), WDRenderUtils::renderUser($user));
				break;	
			case 'ADMIN_RESIGNED':
				$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($extra['user_id']);
				$body = sprintf(_('The user %s resigned from being an administrator of this site.'), WDRenderUtils::renderUser($user));
				break;	
		}
		
		return $body;
	}
	
	public function getUrls(){
		$type = $this->getType();
		$extra = $this->getExtra();
		if($extra['urls']){
			return 	$extra['urls'];
		}
		$lang = OZONE::getRunData()->getLanguage();
		$site = OZONE::getRunData()->getTemp("site");
		
		switch($type){
			case 'NEW_MEMBER_APPLICATION':
				$urls =  array(	array(_('check pending applications'),'http://'.$site->getDomain().'/admin:manage/start/ma')
								);
				break;	
			case 'INVITATION_ACCEPTED':
				$urls  = array(	array(_('site members'),'http://'.$site->getDomain().'/admin:manage/start/members-list')
								);
				break;	
			case 'INVITATION_DECLINED':
				break;	
			case 'NEW_MEMBER_BY_PASSWORD':
				$urls = array(	array('_(site members)','http://'.$site->getDomain().'/admin:manage/start/members-list')
							);
				break;	
			case 'MEMBER_RESIGNED':
				$urls = array(	array(_('site members'),'http://'.$site->getDomain().'/admin:manage/start/members-list')
								);
				break;	
			case 'MODERATOR_RESIGNED':
				$urls = array(	array(_('site moderators'),'http://'.$site->getDomain().'/admin:manage/start/moderators'),
					array(_('site members'),'http://'.$site->getDomain().'/admin:manage/start/members-list')
								);
				break;	
			case 'ADMIN_RESIGNED':
				$urls = array(	array(_('site adminitrators'),'http://'.$site->getDomain().'/admin:manage/start/admins'),
					array(_('site members'),'http://'.$site->getDomain().'/admin:manage/start/members-list')
								);
				break;	
		}
		return $urls;
	}
	
}
