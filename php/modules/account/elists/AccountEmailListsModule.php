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

class AccountEmailListsModule extends AccountBaseModule {
	
	public function build($runData){
		
		$pl = $runData->getParameterList();
		$totalAll = (bool) $pl->getParameterValue('totalAll');
		
		$user = $runData->getUser();
		$c = new Criteria();
		if($totalAll){
			$q = "SELECT site.* FROM site, member WHERE member.user_id = '{$user->getUserId()}' AND member.site_id = site.site_id " .
					"ORDER BY site.name";
			$c->setExplicitQuery($q);
			$ss = DB_SitePeer::instance()->select($c);
			$sites = array();
			foreach($ss as $s){
				$sites[$s->getUnixName()] = array('site' => $s);
			}
		}else{
			$q = "SELECT email_list.* FROM email_list, email_list_subscriber, site WHERE email_list_subscriber.user_id = {$user->getUserId()} " .
					"AND email_list_subscriber.list_id = email_list.list_id AND email_list.site_id = site.site_id " .
					"ORDER BY site.name, email_list.title";
			$c->setExplicitQuery($q);
			
			$lists = DB_EmailListPeer::instance()->select($c);
			
			// sorry  for the DIIIIRTY STYLE!!!
			$sites = array();
			foreach($lists as $l){
				$s = DB_SitePeer::instance()->selectByPrimaryKey($l->getSiteId());
				if(!isset($sites[$s->getUnixName()])){
					$sites[$s->getUnixName()] = array('site' => $s, 'lists' => array());
				}
				$sites[$s->getUnixName()]['lists'][] = $l;
				$l->setTemp('site', $s);
			}
	}
		$runData->contextAdd('lists', $lists);
		$runData->contextAdd('sites', $sites);
		$runData->contextAdd('totalAll', $totalAll);
		$runData->contextAdd('user', $user);
	}
	
}
