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

class AccountMembershipAction extends SmartyAction {
	
	public static $forbiddenUnixNames = array(
		'/^www[0-9]*$/',
		'/^[0-9]*www$/',
		'/^www\-/',
		'/^community\-/',
		'/^mail$/',
		'/^\-/',
		'/\-$/',
		'/^lab(s)?$/',
		'/^open$/',
		'/^dev$/',
		'/^blog$/',
		'/wikidot/',
		'/^pro$/',
		'/^mail$/',
		'/michalfrackowiak/',
		'/michal\-frackowiak/',
		'/^film$/',
		'/^web$/',
		'/^ssl$/',
		'/^payment[s]?$/',
		'/^pay$/',
		'/^service[s]?$/',
		'/^redbeard$/',
		'/^photo$/',
		'/^img$/',
		'/^fotoforum$/',
		'/^stat[s]?$/',
		'/^your\-?site$/',
		'/^template\-/'
		
		);
	
	public function isAllowed($runData){
		$userId = $runData->getUserId();
		if($userId == null || $userId <1){
			throw new WDPermissionException(_("Not allowed. You should login first."));
		}
		return true;
	}
	
	public function perform($runData){}
	
	public function acceptInvitationEvent($runData){
		$pl = $runData->getParameterList();
		$invitationId = $pl->getParameterValue("invitation_id");
		$userId = $runData->getUserId();
		$user = $runData->getUser();
		
		$invitation = DB_MemberInvitationPeer::instance()->selectByPrimaryKey($invitationId);
		$site = DB_SitePeer::instance()->selectByPrimaryKey($invitation->getSiteId());
		if($invitation == null || $invitation->getUserId() != $userId || $site == null){
			throw new ProcessException(_("Invitation can not be found."), "no_invitation");
		}
		
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
		// all ok... ;-)
		
		$db = Database::connection();
		$db->begin();
		// create membership
		$member = new DB_Member();
		$member->setUserId($userId);
		$member->setSiteId($invitation->getSiteId());
		$member->setDateJoined(new ODate());
		
		$member->save();
		
		$ml = new DB_MembershipLink();
		$ml->setUserId($userId);
		$ml->setSiteId($invitation->getSiteId());
		$ml->setDate(new ODate());
		$ml->setType('INTERNAL_INVITATION');
		$ml->setByUserId($invitation->getByUserId());
		$ml->save();
		
		// remove application (if any) 
		$c = new Criteria();
		$c->add("site_id", $site->getSiteId());
		$c->add("user_id", $userId);
			
		DB_MemberApplicationPeer::instance()->delete($c);
		
		DB_MemberInvitationPeer::instance()->deleteByPrimaryKey($invitationId);
		$runData->ajaxResponseAdd("message", _('Now you are a member of the site').' <a href="http://'.htmlspecialchars($site->getDomain()).'">'.htmlspecialchars($site->getName()).'</a>');
		
		AdminNotificationMaker::instance()->memberInvitationAccepted($site, $user);
		$db->commit();
	}
	
	public function throwAwayInvitationEvent($runData){
		$pl = $runData->getParameterList();
		$invitationId = $pl->getParameterValue("invitation_id");
		$userId = $runData->getUserId();
		$user = $runData->getUser();
		
		$db = Database::connection();
		$db->begin();
		
		$invitation = DB_MemberInvitationPeer::instance()->selectByPrimaryKey($invitationId);
		$site = DB_SitePeer::instance()->selectByPrimaryKey($invitation->getSiteId());
		if($invitation == null || $invitation->getUserId() != $userId || $site == null){
			throw new ProcessException(_("Invitation can not be found."), "no_invitation");
		}
		
		$c = new Criteria();
		$c->add("invitation_id", $invitationId);
		$c->add("user_id", $userId);
		DB_MemberInvitationPeer::instance()->delete($c);
		AdminNotificationMaker::instance()->memberInvitationDeclined($site, $user);
		$db->commit();
	}
	
	public function signOffEvent($runData){
		// remove the membership AND adminship AND moderatorship
		
		$siteId = $runData->getParameterList()->getParameterValue("site_id");
		$userId = $runData->getUserId();
		$user = $runData->getUser();
		$c = new Criteria();
		$c->add("user_id", $userId);
		$c->add("site_id", $siteId);
		
		$db = Database::connection();
		$db->begin();
		
		// check if admin
		
		$admin =  DB_AdminPeer::instance()->selectOne($c);
		
		if($admin && $admin->getFounder()){
			throw new ProcessException(_("You have founded this site - sorry, you can not resign."), "founder_nonremovable");	
		}
		
		if($admin){
			// check if not the last admin!!!
			$c2 = new Criteria();
			$c2->add("site_id", $siteId);
			$acount = DB_AdminPeer::instance()->selectCount($c2);
			if($acount == 1){
				$runData->ajaxResponseAdd("status", "last_admin");
				$runData->ajaxResponseAdd("message", _("You can not simply resign - you are the last admin of this site!"));
				$db->commit();
				return;
			}
		}
		
		DB_MemberPeer::instance()->delete($c);
		DB_ModeratorPeer::instance()->delete($c);
		DB_AdminPeer::instance()->delete($c);
		
		$site = DB_SitePeer::instance()->selectByPrimaryKey($siteId);
		AdminNotificationMaker::instance()->memberResigned($site, $user);
		
		$db->commit();	
	}
	
	public function adminResignEvent($runData){
		$siteId = $runData->getParameterList()->getParameterValue("site_id");
		$userId = $runData->getUserId();
		
		$db = Database::connection();
		$db->begin();
		
		$c = new Criteria();
		$c->add("user_id", $userId);
		$c->add("site_id", $siteId);
		$admin = DB_AdminPeer::instance()->selectOne($c);
		
		if($admin && $admin->getFounder()){
			throw new ProcessException(_("You have founded this site - sorry, you can not resign."), "founder_nonremovable");	
		}
		
		// you can not resign if you are the last admin...
		$c2 = new Criteria();
		$c2->add("site_id", $siteId);
		$acount = DB_AdminPeer::instance()->selectCount($c2);
		if( $acount == 1){
			$runData->ajaxResponseAdd("status", "last_admin");
			$runData->ajaxResponseAdd("message", _("You can not simply resign - you are the last admin of this site!"));
			$db->commit();
			return;
		}
		
		$c = new Criteria();
		$c->add("user_id", $userId);
		$c->add("site_id", $siteId);
		DB_AdminPeer::instance()->delete($c);
		
		$site = DB_SitePeer::instance()->selectByPrimaryKey($siteId);
		$user = $runData->getUser();
		AdminNotificationMaker::instance()->adminResigned($site, $user);
		
		$db->commit();
	}
	
	public function moderatorResignEvent($runData){
		$siteId = $runData->getParameterList()->getParameterValue("site_id");
		$userId = $runData->getUserId();
		
		$db = Database::connection();
		$db->begin();

		$c = new Criteria();
		$c->add("user_id", $userId);
		$c->add("site_id", $siteId);
		DB_ModeratorPeer::instance()->delete($c);
		
		$site = DB_SitePeer::instance()->selectByPrimaryKey($siteId);
		$user = $runData->getUser();
		AdminNotificationMaker::instance()->moderatorResigned($site, $user);
		
		$db->commit();
	}
	
	public function removeApplicationEvent($runData){
		$siteId = $runData->getParameterList()->getParameterValue("site_id");
		$userId = $runData->getUserId();
		
		$db = Database::connection();
		$db->begin();

		$c = new Criteria();
		$c->add("user_id", $userId);
		$c->add("site_id", $siteId);
		DB_MemberApplicationPeer::instance()->delete($c);
		
		$db->commit();
	}
	
	public function restoreSiteEvent($runData){
		$pl = $runData->getParameterList();
		$siteId = $pl->getParameterValue('siteId');
		$unixName = trim($pl->getParameterValue('unixName'));
		
		$c = new Criteria();
		$c->add('site_id', $siteId);
		$c->add('deleted', true);
		$site = DB_SitePeer::instance()->selectOne($c);
		
		if(!$site){
			throw new ProcessException(_('Error selecting a site to restore.'));
		}
	
		// check if allowed
		$user = $runData->getUser();
		
		$c = new Criteria();
		$c->add("user_id", $user->getUserId());
		$c->add("site_id", $site->getSiteId());
		$c->add("founder", true);
		$rel = DB_AdminPeer::instance()->selectOne($c);
		
		if(!$rel){
			throw new ProcessException(_("Sorry, you have no permissions to restore this site."));
		}
		
		$db = Database::connection();
		$db->begin();
		
		// validate unix name
		$errors = array();
		if($unixName === null || strlen($unixName)<3 || strlen(WDStringUtils::toUnixName($unixName)) < 3){
			$errors['unixname'] = _("Web address must be present and should be at least 3 characters long.");	
		}elseif(strlen($unixName)>30){
			$errors['unixname']	 = _("Web address name should not be longer than 30 characters.");
		}elseif(preg_match("/^[a-z0-9\-]+$/", $unixName) == 0){
			$errors['unixname']	= _('Only lowercase alphanumeric and "-" (dash) characters allowed in the web address.');
		}elseif(preg_match("/\-\-/", $unixName) !== 0){
			$errors['unixname']	= _('Only lowercase alphanumeric and "-" (dash) characters allowed in the web address. Double-dash (--) is not allowed.');
		}else{
			
			$unixName = WDStringUtils::toUnixName($unixName);
			
			if(!$runData->getUser()->getSuperAdmin()){
			 	//	handle forbidden names	
			 	$forbiddenUnixNames = explode("\n", file_get_contents(WIKIDOT_ROOT.'/conf/forbidden_site_names.conf'));
				foreach($forbiddenUnixNames as $f){
					if(preg_match($f, $unixName) >0){
						$errors['unixname']	= _('For some reason this web address is not allowed or is reserved for future use.');	
					}	
				}
			}
			
			// check if the domain is not taken.
			$c = new Criteria();
			$c->add("unix_name", $unixName);
			$ss = DB_SitePeer::instance()->selectOne($c);
			if($ss){
				$errors['unixname'] = _('Sorry, this web address is already used by another site.');
						
			}	
			
		}
		
		if(isset($errors['unixname'])){
			throw new ProcessException($errors['unixname']); 
		}

		$oldUnixName = $site->getUnixName();
		$oldLocalPath = $site->getLocalFilesPath();
		$site->setUnixName($unixName);
		// 	rename the files
		mkdirfull(dirname($site->getLocalFilesPath()));
		@rename($oldLocalPath, $site->getLocalFilesPath());
		
		$site->setDeleted(false);
		$site->setCustomDomain(null);
		$site->save();
		
		$db->commit();
		
		$runData->ajaxResponseAdd('unixName', $site->getUnixName());
	}

}
