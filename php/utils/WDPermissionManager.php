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

class WDPermissionManager {
	
	private static $instance;
	
	private $checkIpBlocks = true;
	private $checkUserBlocks = true;
	/**
	 * If set to true each permission check will generate a 
	 * PermissionException instead of returning false if
	 * action is not permitted.
	 */
	private $throwExceptions = true; 
	
	private static $pageActions = array(
			'view' => 'v',
			'edit' => 'e',
			'create' => 'c',
			'move' => 'm',
			'delete' => 'd',
			'attach_file' => 'a',
			'rename_file' => 'r',
			'replace_file' => 'z',
			'move_file' => 'z',
			'delete_file' => 'z',
			'options' => 'o'
		);
		
	private static $pageActionsDesc = array(
			'view' => 'view this page',
			'edit' => 'edit this page',
			'create' => 'create a new page in this category',
			'move' => 'move this page',
			'delete' => 'delete this page',
			'attach_file' => 'attach a new file to this page',
			'rename_file' => 'rename file attachment in this page',
			'replace_file' => 'replace existing file attachment in this page',
			'move_file' => 'move file attachment to another page',
			'delete_file' => 'delete file in this page',
			'options' => 'what????'
		);
	
	private static $forumActions = array(
			'new_thread' => 't',
			'new_post' => 'p',
			'edit_post' => 'e',
			'edit_thread' => 'e',
			'split' => 's',
			'moderate_forum' => 'x'
		);
	private static $forumActionsDesc = array(
			'new_thread' => 'start new discussion thread',
			'new_post' => 'add new post in this thread',
			'edit_post' => 'edit a post in this thread',
			'edit_thread' => 'edit this thread',
			'split' => 's',
			'moderate_forum' => 'perform this action'
		);
	
	private static $userClasses = array(
		'anonymous' => 'a',
		'registered' => 'r',
		'member' => 'm',
		'owner' => 'o'
	);
	
	private static $userClassesDesc = array(
		'anonymous' => 'anonymous users',
		'registered' => 'registered users',
		'member' => 'members of this site',
		'owner' => 'owner (creator) of this page'
	);
	
	public static function instance(){
		if(self::$instance == null){
			self::$instance = new WDPermissionManager();
		}
		return self::$instance;	
	}

	public function __construct(){
		
		self::$pageActionsDesc = array(
			'view' => _('view this page'),
			'edit' => _('edit this page'),
			'create' => _('create a new page in this category'),
			'move' => _('move this page'),
			'delete' => _('delete this page'),
			'attach_file' => _('attach a new file to this page'),
			'rename_file' => _('rename file attachment in this page'),
			'replace_file' => _('replace existing file attachment in this page'),
			'move_file' => _('move file attachment to another page'),
			'delete_file' => _('delete file in this page'),
			'options' => _('what????')
		);
		
		self::$forumActionsDesc = array(
			'new_thread' => _('start new discussion thread'),
			'new_post' => _('add new post in this thread'),
			'edit_post' => _('edit a post in this thread'),
			'edit_thread' => _('edit this thread'),
			'split' => 's',
			'moderate_forum' => _('perform this action'));
			
		self:: $userClassesDesc = array(
			'anonymous' => _('anonymous users'),
			'registered' => _('registered users'),
			'member' => _('members of this site'),
			'owner' => _('owner (creator) of this page'));
	}
	
	public function hasPermission($action, $user, $site=null){

		if($user){
			if((is_string($user) && is_numeric($user)) || is_int($user)){
				if($user >0){
					$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($user);
				} else {
					$user = null;
				}	
			}
		}
		
		if($user && ($user->getSuperAdmin() || $user->getSuperModerator() )){
			return true;	
		}
		
		if(($site && is_string($site) && is_numeric($site)) || is_int($site)){
			$site = DB_SitePeer::instance()->selectByPrimaryKey($site);	
		}
		 
		if($site && $site->getDeleted()){
			$message = _("This site has been deleted.");
			$action = null;
		}
		
		switch($action){
			/*
			 * check for site management permissions. 
			 * user must be in the admin group.
			 */
			
			case 'manage_site':
				if(!$user){
					$message = _("You have no permission to configure the site properties. Only site admins are allowed to do it.".
					"But right now you are not even logged in...");
				}else{
					$c = new Criteria();
					$c->add("user_id", $user->getUserId());
					$c->add("site_id", $site->getSiteId());
					$rel = DB_AdminPeer::instance()->selectOne($c);
					if($rel == null){
						$message = _("You have no permission to configure the site properties. Only site admins are allowed to do it.");
					}
				}
				break;
			case 'account':
				if(!$user){
					$message = _("You can not access your account while not being logged in...");
				}
				break;
			case 'become_member':
				
				$c = new Criteria();;
				$c->add("user_id", $user->getUserId());
				
				$mc = DB_MemberPeer::instance()->selectCount($c);
				
				// count memberships
					
				// check if not on a blacklist
				$block = $this->checkUserBlocked($user, $site);
				if($block){
					$message = _("Sorry, you are banned from this site and not allowed to join.");
					if($block->getReason() && $block->getReason()!=''){
						$message .=	_("The given reason is:")." <p>".htmlspecialchars($block->getReason())."</p>";
					}
				}
				break;
		}

		if($message){
			// shit. no permission.
			if($this->throwExceptions == true){
				// throw exception
				throw new WDPermissionException($message);		
			}
			else{
				return false;	
			}
		}else{
			return true;
		}
		
	}

	public function hasPagePermission($action, $user, $category, $page=null, $site=null){
		if($user){
			if((is_string($user) && is_numeric($user)) || is_int($user)){
				$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($user);	
			}
		}
		
		if($user && ($user->getSuperAdmin() || $user->getSuperModerator())){
			return true;	
		}
		
		$site = $GLOBALS['site']; // ugly.
		
		// ban by IP first.	
		if($this->checkIpBlocks){
			$ips = Ozone::getRunData()->createIpString();			
			$blocks = $this->checkIpBlocked($ips, $site);
			if(count($blocks)>0){
				if($this->throwExceptions){
					throw new WDPermissionException(_("Sorry, your IP address is blocked from participating in and modifying this site."));
				}else{
					return false;
				}
			}
		}
		
		// check if page not blocked
		if($page && $page instanceof DB_Page && $page->getBlocked()){
			if($user){
				// still nothing. check if moderator of "pages".
				$c = new Criteria();
				$c->add("site_id", $category->getSiteId());
				$c->add("user_id", $user->getUserId());
				$rel = DB_ModeratorPeer::instance()->selectOne($c);
				if($rel && strpos($rel->getPermissions(), 'p') !== false){
					return true;
				}
				
				// still nothing. check if admin.
				$c = new Criteria();
				$c->add("site_id", $category->getSiteId());
				$c->add("user_id", $user->getUserId());
				$rel = DB_AdminPeer::instance()->selectOne($c);
				if($rel){
					return true;
				}
			}
			// if not - can not edit!
			throw new WDPermissionException(_("This page is blocked and only Site Administrators and Moderators with enough privileges can modify it."));
		}

		//action code
		$ac = self::$pageActions[$action];
		//permission string
		$ps = $category->getPermissionString();
		
		// first try anonymous and registered to save effort
		$uc = self::$userClasses['anonymous'];
		if($this->permissionLookup($ac, $uc, $ps)){
			// ok, anyone can.
			// but check ip blocks.
			if($this->checkUserBlocks && $user){
				
				$block = $this->checkUserBlocked($user, $site);
				if($block){
					if($this->throwExceptions){
						$message = _("Sorry, you are blocked from participating in and modifying this site. ");
						if($block->getReason() && $block->getReason()!=''){
							$message .=	_("The given reason is:")." <p>".htmlspecialchars($block->getReason())."</p>";
						}
						throw new WDPermissionException($message);
					}else{
						return false;
					}	
				}	
			}
			return true;		
		}elseif(!$user){
			// anonymous can not and the user is only anonymous. game over.
	
			$m = $this->generateMessage($action, $uc, $ps);
			$this->handleFalse($m);
			return false;
		}
			
		// ok, check registered now
		$uc = self::$userClasses['registered'];
		if($this->permissionLookup($ac, $uc, $ps)){
			// check blocked users
			if($this->checkUserBlocks){
				$block = $this->checkUserBlocked($user, $site);
				if($block){
					if($this->throwExceptions){
						$message = _("Sorry, you are blocked from participating in and modifying this site. ");
						if($block->getReason() && $block->getReason()!=''){
							$message .=	_("The given reason is:")." <p>".htmlspecialchars($block->getReason())."</p>";
						}
						throw new WDPermissionException($message);
					//	throw new WDPermissionException("Sorry, you are blocked from participating in and modifying this site. " .
					}else{
						return false;
					}	
				}
			}
			return true;
		}
			
		// ok, a "premium feature" or what... need to check members now...
		$uc = self::$userClasses['member'];
		if($this->permissionLookup($ac, $uc, $ps)){
				
			// ok, members CAN do this. is the user a member?
			$c = new Criteria();
			$c->add("site_id", $category->getSiteId());
			$c->add("user_id", $user->getUserId());
			$rel = DB_MemberPeer::instance()->selectOne($c);
			if($rel){
				return true;
			}
		}
		
		$uc = self::$userClasses['owner'];
		if($page && $this->permissionLookup($ac, $uc, $ps)){
			if($site && is_string($page)){
				$page = DB_PagePeer::instance()->selectByName($site->getSiteId(), $page);	
			}
			if($page && $page->getOwnerUserId() && $user->getUserId() == $page->getOwnerUserId()){
				// check blocked users
				if($this->checkUserBlocks){
					$block = $this->checkUserBlocked($user, $site);
					if($block){
						if($this->throwExceptions){
							$message = _("" .
									"Sorry, you are blocked from participating in and modifying this site. ");
							if($block->getReason() && $block->getReason()!=''){
								$message .=	_("The given reason is:")." <p>".htmlspecialchars($block->getReason())."</p>";
							}
							throw new WDPermissionException($message);
						}else{
							return false;
						}	
					}
				}
				return true;
			}	
		}
		
		// still nothing. check if moderator of "pages".
		$c = new Criteria();
		$c->add("site_id", $category->getSiteId());
		$c->add("user_id", $user->getUserId());
		$rel = DB_ModeratorPeer::instance()->selectOne($c);
		if($rel && strpos($rel->getPermissions(), 'p') !== false){
			return true;
		}
		
		// still nothing. check if admin.
		$c = new Criteria();
		$c->add("site_id", $category->getSiteId());
		$c->add("user_id", $user->getUserId());
		$rel = DB_AdminPeer::instance()->selectOne($c);
		if($rel){
			return true;
		}
		
		$m = $this->generateMessage($action, $uc, $ps);
		$this->handleFalse($m);
		return false;
		
	}
	
	public function hasForumPermission($action, $user, $category, $thread=null, $post=null){
		if($user){
			if((is_string($user) && is_numeric($user)) || is_int($user)){
				$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($user);	
			}
		}
		
		if($user && ($user->getSuperAdmin() || $user->getSuperModerator())){
			return true;	
		}
		
		$site = $GLOBALS['site']; // ugly.
		
		// ban by IP first.	
		if($this->checkIpBlocks){
			$ips = Ozone::getRunData()->createIpString();			
			$blocks = $this->checkIpBlocked($ips, $site);
			if(count($blocks)>0){
				if($this->throwExceptions){
					throw new WDPermissionException(_("Sorry, your IP address is blocked from participating in and modifying this site."));
				}else{
					return false;
				}
			}
		}
		
		if(strpos($action, "thread")){
			$authorString = _("author of the thread");	
		}else {
			$authorString = _("author of the post");	
		}
		
		//action code
		$ac = self::$forumActions[$action];
		//permission string
		$ps = $category->getPermissionString();

		// first try anonymous and registered to save effort
		$uc = self::$userClasses['anonymous'];
		if($this->permissionLookup($ac, $uc, $ps)){
			// ok, anyone can.
			// but check ip blocks.
			if($this->checkUserBlocks && $user){
				
				$block = $this->checkUserBlocked($user, $site);
				if($block){
					if($this->throwExceptions){
						$message = _("Sorry, you are blocked from participating in and modifying this site. ");
						if($block->getReason() && $block->getReason()!=''){
							$message .=	_("The given reason is:")." <p>".htmlspecialchars($block->getReason())."</p>";
						}
						throw new WDPermissionException($message);
						//throw new WDPermissionException("Sorry, you are blocked from participating in and modifying this site. " .
					}else{
						return false;
					}	
				}	
			}
			return true;		
		}elseif(!$user){
			// anonymous can not and the user is only anonymous. game over.
			$m = $this->generateMessage($action, $uc, $ps, 'forum', array("o" => $authorString));
			$this->handleFalse($m);
			return false;
		}
			
		// ok, check registered now
		$uc = self::$userClasses['registered'];
		if($this->permissionLookup($ac, $uc, $ps)){
			// check blocked users
			if($this->checkUserBlocks){
				$block = $this->checkUserBlocked($user, $site);
				if($block){
					if($this->throwExceptions){
						$message = _("Sorry, you are blocked from participating in and modifying this site. ");
						if($block->getReason() && $block->getReason()!=''){
							$message .=	_("The given reason is:")." <p>".htmlspecialchars($block->getReason())."</p>";
						}
						throw new WDPermissionException($message);
						//throw new WDPermissionException("Sorry, you are blocked from participating in and modifying this site. " .
					}else{
						return false;
					}	
				}
			}
			return true;
		}
		
		// ok, a "premium feature" or what... need to check members now...
		$uc = self::$userClasses['member'];
		if($this->permissionLookup($ac, $uc, $ps)){
			// ok, members CAN do this. is the user a member?
			$c = new Criteria();
			$c->add("site_id", $category->getSiteId());
			$c->add("user_id", $user->getUserId());
			$rel = DB_MemberPeer::instance()->selectOne($c);
			if($rel){
				return true;
			}
		}
		
		$uc = self::$userClasses['owner'];
		if(($post || $thread) && $this->permissionLookup($ac, $uc, $ps)){
			$o = $post ? $post:$thread;
			
			if($o && $o->getUserId() && $user->getUserId() == $o->getUserId()){
				// check blocked users
				if($this->checkUserBlocks){
					$block = $this->checkUserBlocked($user, $site);
					if($block){
						if($this->throwExceptions){
							$message = _("Sorry, you are blocked from participating in and modifying this site. ");
							if($block->getReason() && $block->getReason()!=''){
								$message .=	_("The given reason is:")." <p>".htmlspecialchars($block->getReason())."</p>";
							}
							throw new WDPermissionException($message);
							//throw new WDPermissionException("Sorry, you are blocked from participating in and modifying this site. " .
						}else{
							return false;
						}	
					}
				}
				return true;
			}	
		}

		// still nothing. check if moderator of "forum".
		$c = new Criteria();
		$c->add("site_id", $category->getSiteId());
		$c->add("user_id", $user->getUserId());
		$rel = DB_ModeratorPeer::instance()->selectOne($c);
		if($rel && strpos($rel->getPermissions(), 'f') !== false){
			return true;
		}
		
		// still nothing. check if admin.
		$c = new Criteria();
		$c->add("site_id", $category->getSiteId());
		$c->add("user_id", $user->getUserId());
		$rel = DB_AdminPeer::instance()->selectOne($c);
		if($rel){
			return true;
		}
		
		$m = $this->generateMessage($action, $uc, $ps, 'forum', array("o" => $authorString));
		$this->handleFalse($m);
		return false;
		
	}
	
	/**
	 * Check if $user can send a private message to $toUser.	
	 */
	public function hasPmPermission($user, $toUser){
		
		if($user && ($user->getSuperAdmin() || $user->getSuperModerator())){
			return true;	
		}
		
		// first check if if $user has pm enabled
		$us = DB_UserSettingsPeer::instance()->selectByPrimaryKey($toUser->getUserId());
		$p = $us->getReceivePm();
		if($this->isUserSuperior($user, $toUser)){
			return true;	
		}
		
		// accept from none (unless from moderators/admins of common sites)
		
		if($p == 'n'){
			throw new WDPermissionException(_("This user does wish to receive private messages."));
		}
		
		if($p == 'mf'){
			if($this->shareSites($user, $toUser)){
				// so they share common sites. check for blocks!	
				if($this->userBlocksUser($toUser, $user)){
					throw new WDPermissionException(_("You are blocked by this user."));
				}
				return true;
			}
			
			// if friends - return true (todo)
			$c = new Criteria();
			$c->add("user_id", $user->getUserId());
			$c->add("target_user_id", $toUser->getUserId());
			$con = DB_ContactPeer::instance()->selectOne($c);
			if($con){
				return true;
			}
			
			throw new WDPermissionException(_("This user wishes to receive messages only from selected users."));
				
		}
		
		if($p == 'f'){
			// check if a friend
			$c = new Criteria();
			$c->add("user_id", $toUser->getUserId());
			$c->add("target_user_id", $user->getUserId());
			$con = DB_ContactPeer::instance()->selectOne($c);
			if($con){
				return true;
			}
			throw new WDPermissionException(_("This user wishes to receive messages only from selected users."));
		}
		
		if($p == 'a'){
			// check if not blocked
			if($this->userBlocksUser($toUser, $user)){
				throw new WDPermissionException(_("You are blocked by this user."));
			}	
		}
		
		// in any other case check 
		return true;
		
	}
	
	public function canBecomeAdmin($user){
		
		if($user->getSuperAdmin()){
			return true;
		}
		
		// check how many sites does the user administer.
		
		$c = new Criteria();;
		$c->add("user_id", $user->getUserId());
		$c->addJoin("site_id", "site.site_id");
		$c->add("site.deleted", false);
		
		$ac = DB_AdminPeer::instance()->selectCount($c);
		$us = $user->getSettings();
		if($ac >= $us->getMaxSitesAdmin()){
			throw new WDPermissionException(sprintf(_("Sorry, a single User can administer max %d Sites at the moment."),$us->getMaxSitesAdmin()));	
		}
		return true;
	}
	
	private function permissionLookup($actionCode, $userCode, $permString){
		$p = "/$actionCode:[a-z]*$userCode/";
		if(preg_match($p, $permString) == 0){
			return false;
		}else{
			return true;
		}
		
	}
	
	private function handleFalse($message = null){
		if(!$this->throwExceptions){
			return false;
		}else{
			if($message == null){
				$message = "No permission";
			}
			throw new WDPermissionException($message);
		}	
	} 
	
	private function generateMessage($ac, $uc, $permString, $mode='page', $extraUserDesc = null){
		if($mode == 'page'){
			$actionString = self::$pageActionsDesc[$ac];
			$actionCode =  self::$pageActions[$ac];
		}elseif($mode=='forum'){
			$actionString = self::$forumActionsDesc[$ac];
			$actionCode =  self::$forumActions[$ac];	
		}
		$allowedUsers = array();
		
		if(preg_match("/^.*?$actionCode:([a-z]*).*$/", $permString) !== 0){
			$a2 = preg_replace("/^.*?$actionCode:([a-z]*).*$/", "\\1", $permString);
			for($i = 0; $i<strlen($a2); $i++){
				if($extraUserDesc && $extraUserDesc[$a2{$i}]){
					$allowedUsers[] = $extraUserDesc[$a2{$i}];
				}else{
					$allowedUsers[] = self::$userClassesDesc[array_search($a2{$i}, self::$userClasses)];
				}	
			}
		}
		$allowedUsers[] = _('site administrators and perhaps selected moderators');
		$m = _('Sorry, you can not ').' '.$actionString.'. ' .
				_('Only ').' '.implode(', ', $allowedUsers).' '._(' are allowed to.');
		return $m;
	}
	
	public function setThrowExceptions($val){
		$this->throwExceptions = $val;	
	}
	
	private function shareSites($user1, $user2){
		$db = Database::connection();
		$q = "SELECT member.* FROM member, member AS member2 WHERE member.user_id = '".$user1->getUserId()."' " .
					"AND member2.user_id = '".$user2->getUserId()."' " .
					"AND member.site_id = member2.site_id " .
					"LIMIT 1";
		$r = $db->query($q);
		if($r->nextRow()){
			return true;
		}else{
			return false;
		}
	}
	/**
	 * Check if user1 is an admin/mod of one of the sites user2 is a member of.
	 */
	private function isUserSuperior($user1, $user2){
		$db = Database::connection();
		$q = "SELECT member.* FROM member, admin WHERE member.user_id = '".$user2->getUserId()."' " .
					"AND admin.user_id = '".$user1->getUserId()."' " .
					"AND member.site_id = admin.site_id " .
					"LIMIT 1";
		$r = $db->query($q);
		if($r->nextRow()){
			return true;	
		}
		$q = "SELECT member.* FROM member, moderator WHERE member.user_id = '".$user2->getUserId()."' " .
					"AND moderator.user_id = '".$user1->getUserId()."' " .
					"AND member.site_id = moderator.site_id " .
					"LIMIT 1";
		$r = $db->query($q);
		if($r->nextRow()){
			return true;	
		}
		return false;
	}
	
	/**
	 * Checks if user1 blocks user2.
	 */
	private function userBlocksUser($user1, $user2){
		$c = new Criteria();
		$c->add("user_id", $user1->getUserId());
		$c->add("blocked_user_id", $user2->getUserId());
		$b = DB_PrivateUserBlockPeer::instance()->selectOne($c);
		if($b !== null){
			return true;
		}else{
			return false;
		}
	}
	
	public function setCheckIpBlocks($val){
		$this->checkIpBlocks = $val;		
	}
	
	public function setCheckUserBlocks($val){
		$this->checkUserBlocks = $val;		
	}
	
	private function checkIpBlocked($ipString, $site){
		$c = new Criteria();
		
		$ips = explode("|", $ipString);
		$q = "SELECT * FROM ip_block WHERE site_id='".$site->getSiteId()."' " .
				"AND (ip <<= '".db_escape_string($ips[0])."' ";
		if($ips[1]) { $q.=	"OR ip <<= '".db_escape_string($ips[1])."'";}
		$q .= ")";
		$c->setExplicitQuery($q);	
		$blocks = DB_IpBlockPeer::instance()->select($c);
		return $blocks;
	}
	
	private function checkUserBlocked($user, $site){
		$c = new Criteria();
		
		$c->add("site_id", $site->getSiteId());
		$c->add("user_id", $user->getUserId());
		$block = DB_UserBlockPeer::instance()->selectOne($c);
		return $block;
	}
	
}
