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

class AccountEmailListsFromSiteModule extends AccountBaseModule {
	
	public function build($runData){
		$user = $runData->getUser();
		$c = new Criteria();
		
		$pl = $runData->getParameterList();
		$siteId = $pl->getParameterValue('siteId');
		
		$all = (bool) $pl->getParameterValue('all');
		
		$site = DB_SitePeer::instance()->selectByPrimaryKey($siteId);
		if($all){
			$q = "SELECT email_list.* FROM email_list WHERE " .
					"email_list.site_id = '{$site->getSiteId()}' " .
					"ORDER BY email_list.title";
			$c->setExplicitQuery($q);
		
			$lists = DB_EmailListPeer::instance()->select($c);
			// check if subscribed
			foreach($lists as $list){
				$c2 = new Criteria();	
				$c2->add('user_id', $user->getUserId());
				$c2->add('list_id', $list->getListId());
				$sub = DB_EmailListSubscriberPeer::instance()->selectOne($c2);
				if($sub){
					$list->setTemp('subscribed', true);
				}
			}
		}else{
			// only subscribed
			$q = "SELECT email_list.* FROM email_list, email_list_subscriber WHERE email_list_subscriber.user_id = {$user->getUserId()} " .
				"AND email_list_subscriber.list_id = email_list.list_id AND email_list.site_id = '{$site->getSiteId()}' " .
				"ORDER BY email_list.title";
			$c->setExplicitQuery($q);
		
			$lists = DB_EmailListPeer::instance()->select($c);
			foreach($lists as $list){
				$list->setTemp('subscribed', true);
			}
		}

		$runData->contextAdd('all', $all);
		$runData->contextAdd('lists', $lists);
		$runData->contextAdd('site', $site);

	}
	
}
