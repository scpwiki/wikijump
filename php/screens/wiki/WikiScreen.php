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

class WikiScreen extends Screen {
	
	private $vars = array();
	
	public function render($runData){
		// get site
		$site = $runData->getTemp("site");
		$runData->contextAdd("site", $site);
		
		$this->handleNotifications($runData);
		
		$pl = $runData->getParameterList();
		
		$wikiPage = $pl->getParameterValue("wiki_page");
		
		$privateAccessGranted = true;
		// check if the site is private
		if($site->getPrivate()){
			$user = $runData->getUser();
			if($user && !$user->getSuperAdmin() && !$user->getSuperModerator()){
				// check if member
				$c = new Criteria();
				$c->add("site_id", $site->getSiteId());
				$c->add("user_id", $user->getUserId());
				$mem = DB_MemberPeer::instance()->selectOne($c);
				if(!$mem) { 
					// check if a viewer
					$c = new Criteria();
					$c->add("site_id", $site->getSiteId());
					$c->add("user_id", $user->getUserId());
					$vi = DB_SiteViewerPeer::instance()->selectOne($c);
					if(!$vi) { 
						$user = null;
					}
				}
			}
			if($user == null){
				$wikiPage = $site->getSettings()->getPrivateLandingPage();
				$privateAccessGranted = false;
			}
		}
		
		if($wikiPage==""){$wikiPage=$site->getDefaultPage();}
		$wikiPage = WDStringUtils::toUnixName($wikiPage);
		$runData->setTemp("pageUnixName", $wikiPage);

		$memcache = Ozone::$memcache;
		if($runData->getAction() == null 
				&& $runData->getRequestMethod() == "GET" 
			
				&& $privateAccessGranted
			){
			// try to get content from the memorycache server
			
			$mcKey = 'page..'.$site->getUnixName().'..'.$wikiPage;
			
			if(strpos( $wikiPage, ":") != false){
				$tmp0 = explode(':',$wikiPage); 
				$categoryName = $tmp0[0];
			} else {
				$categoryName = "_default";
			}
			$aKey = 'category_lc..'.$site->getUnixName().'..'.$categoryName;
			$changeTime = $memcache->get($aKey); 
			$cachedPage = $memcache->get($mcKey); 
			if($cachedPage !== false && $changeTime && $changeTime <= $cachedPage['timestamp']){
				$runData->setTemp("page", $cachedPage['page']);
				$GLOBALS['page'] = $cachedPage['page'];

				$out = $cachedPage['content'];
				if($this->vars['notificationsDialog']){
			
				$out = preg_replace(';<div id="account-notifications-dummy" style="display:none"></div>;', 
						'<div id="notifications-dialog" style="display:none">'.
							$this->vars['notificationsDialog'].'</div>', $out, 1);
				}
				return $out;	
			} else {
				$storeLater = true;	
				
			}	
		}
		
		$runData->contextAdd("wikiPageName", $wikiPage);
		
		$settings = $site->getSettings();
		
		// get wiki page from the database
		$page = DB_PagePeer::instance()->selectByName($site->getSiteId(), $wikiPage);
		
		if($page == null){
			$runData->contextAdd("pageNotExists", true);
			// get category based on suggested page name
			
			if(strpos( $wikiPage, ":") != false){
				$tmp0 = explode(':',$wikiPage); 
				$categoryName = $tmp0[0];
			} else {
				$categoryName = "_default";
			}
			$category = DB_CategoryPeer::instance()->selectByName($categoryName, $site->getSiteId());
			if($category == null){
				$category = DB_CategoryPeer::instance()->selectByName('_default', $site->getSiteId());
			}	
			$runData->setTemp("category", $category);
		} else{
			// page exists!!! wooo!!!	
			
			$runData->setTemp("page", $page);
			$GLOBALS['page'] = $page;
			
			$compiled = $page->getCompiled();
			
			$runData->contextAdd("wikiPage", $page);
			$runData->contextAdd("pageContent", $compiled->getText());

			$category = $page->getCategory();
			$runData->setTemp("category", $category);
			
			// show options?
			$showPageOptions = true;
			$runData->contextAdd("showPageoptions", $showPageOptions);

			// get the tags
			$c = new Criteria();
			$c->add("page_id", $page->getPageId());
			$c->addOrderAscending("tag");
			$tags = DB_PageTagPeer::instance()->select($c);
			$t2 = array();
			foreach($tags as $t){
				$t2[] = $t->getTag();	
			}	
			$runData->contextAdd("tags", $t2);
			
			// has discussion?
			if($page->getThreadId()!== null){
				$thread = DB_ForumThreadPeer::instance()->selectByPrimaryKey($page->getThreadId());
				if($thread == null){
					$page->setThreadId(null);
					$page->save();
				}else{
					$page->setTemp("numberPosts", $thread->getNumberPosts());
				} 	
			}

			// look for parent pages (and prepare breadcrumbs)
			if($page->getParentPageId()){
				$breadcrumbs = array();
				$ppage = DB_PagePeer::instance()->selectByPrimaryKey($page->getParentPageId());
				array_unshift($breadcrumbs, $ppage);
				$bcount = 0;
				while($ppage->getParentPageId() && $bcount<=4){
					$ppage = DB_PagePeer::instance()->selectByPrimaryKey($ppage->getParentPageId());
					array_unshift($breadcrumbs, $ppage);
					$bcount++;
				}
				$runData->contextAdd("breadcrumbs", $breadcrumbs);
			}
		}
		
		$runData->contextAdd("category", $category);
		
		// GET THEME for the category
		
		$theme = $category->getTheme();
		$runData->contextAdd("theme", $theme);
		
		// GET LICENSE for the category
		
		$licenseText = $category->getLicenseText();
		$runData->contextAdd("licenseText", $licenseText);
		
		// show nav elements?
		
		if($privateAccessGranted || !$settings->getHideNavigationUnauthorized()){
	
			if($theme->getUseSideBar()){
				$sideBar1 = $category->getSidePage();
				if($sideBar1 !== null){
					$sideBar1Compiled = $sideBar1->getCompiled();
					$ccc =  $sideBar1Compiled->getText();
					$ccc = preg_replace('/id="[^"]*"/', '', $ccc);
					$runData->contextAdd("sideBar1Content",$ccc);
				}
			}
			if($theme->getUseTopBar()){
				$topBar = $category->getTopPage();
				if($topBar !== null){
					$topBarCompiled = $topBar->getCompiled();
					$ccc =  $topBarCompiled->getText();
					$ccc = preg_replace('/id="[^"]*"/', '', $ccc);
					$runData->contextAdd("topBarContent", $ccc);
				}
			}
		}

		// OpenID stuff now !!!
		
		if($settings->getOpenidEnabled() && $page){
			// find a page
			$c = new Criteria();	
			$c->add("site_id", $site->getSiteId());
			
			if($_SERVER['REQUEST_URI'] == "/"){
				$c->add("page_id", null);			
			}else{
				$c->add("page_id", $page->getPageId());
			}

			$oentry = DB_OpenidEntryPeer::instance()->selectOne($c);
			
			if($oentry){
			
				$openId = array();
				$openId['enabled'] = true;
				$openId['identity'] = $oentry->getUrl();
				$openId['server'] = $oentry->getServerUrl();
				$runData->contextAdd("openId", $openId);
			}
		}

		// check wether to include a special JS file for custom domains or a special JS file for private files
		//if (preg_match('/^([a-zA-Z0-9\-]+)\.' . GlobalProperties::$URL_DOMAIN_PREG . '$/',$_SERVER["HTTP_HOST"], $matches) !==1) {
		//	$runData->contextAdd("useCustomDomainScript", true);
		//}
		
		$smarty = Ozone::getSmarty();
		
		// put context into context
	 	
	 	$context = $runData->getContext();
	 	if($context !== null){
	 		foreach($context as $key => $value){
		 		$smarty->assign($key, $value);
	 		}
	 	}
	 	
	 	$templateFile = PathManager::screenTemplate("wiki/WikiScreen");
	 	$screenContent = $smarty->fetch($templateFile);
	 	
	 	$smarty->assign("screen_placeholder", $screenContent);
	 	$layoutFile = PathManager::layoutTemplate("WikiLayout");
	 	$out = $smarty->fetch($layoutFile);
	 	
		if($storeLater){
			$now = time();
			if(!$changeTime){
				$memcache->set($aKey, $now, 0, 864000);	
			}
			$memcache->set($mcKey, array("page" =>$page, "content" => $out, "timestamp" => $now), 0, 864000);	
		}
	 	
	 	if($this->vars['notificationsDialog']){
			$out = preg_replace(';<div id="account-notifications-dummy" style="display:none"></div>;', 
						'<div id="notifications-dialog" style="display:none">'.
							$this->vars['notificationsDialog'].'</div>', $out, 1);
		}
	 	
	 	return $out;
	}
	
	private function handleNotifications($runData){
		// check not earlier than 2 minutes after the previous check
		$user = $runData->getUser();
		if($user == null){
			return;
		}
		
		// get last check date
		$lastCheck = $_COOKIE['lastncheck'];
		if($lastCheck !== null && is_numeric($lastCheck) && time() - $lastCheck < 120){
			return;	
		}
		
		$cookieResult = setcookie('lastncheck', time(), time() + 10000000, "/", GlobalProperties::$SESSION_COOKIE_DOMAIN);
		// ok. go get the notifications now.
		
		$c = new Criteria();
		$c->add("user_id", $user->getUserId());
		$c->add("notify_online", true);
		$c->addOrderDescending("notification_id");
		
		$nots = DB_NotificationPeer::instance()->select($c);
		
		if(count($nots) == 0){
			return;	
		}
		
		if(count($nots)>0){
			
			$q = "UPDATE notification SET notify_online=FALSE, notify_email=FALSE " .
					"WHERE user_id='".$user->getUserId()."' AND " .
					"notify_online = TRUE";
			$db = Database::connection();
			$db->query($q);
		}
		
		$nots2 = array();
		
		foreach($nots as &$not){
			if($not->getType() == "new_private_message"){
				
				// check if the message is read or still new
				$extra = $not->getExtra();
				$pm = DB_PrivateMessagePeer::instance()->selectByPrimaryKey($extra['message_id']);
				if($pm && $pm->getFlagNew()){
					$body = $not->getBody();
					$body = preg_replace('/<br\/>Preview.*$/sm', '', $body);
					$body = preg_replace(';You have.*?<br/>;sm', '', $body);
					$not->setBody($body);
					$nots2[] = $not;	
				}
			}else{
				$nots2[] = $not;	
			}
			
		}
		
		if(count($nots2)==0){
			return;	
		}
		
		$lang = $user->getLanguage();
		
		switch($lang){
			case 'pl':
				$glang="pl_PL";
				$wp = "pl";
				break;
			case 'en':
				$glang="en_US";
				$wp = "www";
				break;
		}
			
		$runData->setLanguage($lang);
		putenv("LANG=$glang"); 
		putenv("LANGUAGE=$glang"); 
		setlocale(LC_ALL, $glang.'.UTF-8');
		
		// get Smarty and render a dialog
		$smarty = Ozone::getSmartyPlain();
		$dialogTemplateFile  = PathManager::screenTemplate("NotificationDialog");

		$count = count($nots2);
		if($count>3){
			$nots2	= array_slice($nots2, 0, 3);
			$smarty->assign("more", $count -3);
		}
		$smarty->assign("count", $count);
		
		$smarty->assign("notifications", $nots2);
		
		$out = $smarty->fetch($dialogTemplateFile);
		
		$this->vars['notificationsDialog'] = $out;

		$lang = $GLOBALS['lang'];
				
		switch($lang){
			case 'pl':
				$glang="pl_PL";
				break;
			case 'en':
				$glang="en_US";
				break;
		}
				
		$runData->setLanguage($lang);	
		putenv("LANG=$glang"); 
		putenv("LANGUAGE=$glang"); 
		setlocale(LC_ALL, $glang.'.UTF-8');
	}

}
