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

class AdminNotificationsFeed extends FeedScreen {
	
	protected $requiresAuthentication = true;
	
	public function render($runData){
		$user = $runData->getTemp("user");
		$site = $runData->getTemp("site");
		
		// check if site admin
		
		$c = new Criteria();
		$c->add("site_id", $site->getSiteId());
		$c->add("user_id", $user->getUserId());
		
		$admin = DB_AdminPeer::instance()->selectOne($c);
		
		if($admin == null){
			return _("Sorry, you are not allowed to view this feed.");	
		}
		
		$key = "adminnotificationsfeed..".$site->getSiteId();
		$mc = OZONE::$memcache;
		$out = $mc->get($key);
		if($out){
			return $out;	
		}
		$out = parent::render($runData);
		$mc->set($key, $out, 0, 3600);
		return $out;
	}
	
	public function build($runData){
	
		$site = $runData->getTemp("site");
		
		// now just get notifications for the site...
		
		$c = new Criteria();
		$c->add("site_id", $site->getSiteId());
		$c->addOrderDescending('notification_id');
		$c->setLimit(20);	
		
		$nots = DB_AdminNotificationPeer::instance()->select($c);
		
		$channel['title'] = _('Admin notifications for site').' "'.htmlspecialchars($site->getName()).'"';
		$channel['link'] = "http://".$site->getDomain()."/admin:manage/start/notifications";
		
		$items = array();
		
		foreach($nots as $not){
			$extra = $not->getExtra();
			$item = array();
			
			$item['title'] = $not->getTitle();
			switch($not->getType()){
				case "NEW_MEMBER_APPLICATION":
					$item['link'] = "http://".$site->getDomain()."/admin:manage/start/ma";
					break;	
				
				default:
					$item['link'] = "http://".$site->getDomain()."/admin:manage/start/notifications"."#notification-".$not->getNotificationId();;
			}
			
			$body = $not->getBody();
			
			$body = preg_replace('/onclick="[^"]+"/', '', $body);
			
			$item['description'] = $body;

			$item['guid'] = $channel['link']."#notification-".$not->getNotificationId();
			$item['date'] = date('r', $not->getDate()->getTimestamp());
			// TODO: replace relative links with absolute links!
			$content =  '';

			$items[] = $item;	
		}
		
		$runData->contextAdd("channel", $channel);
		$runData->contextAdd("items", $items);
	}
	
}
