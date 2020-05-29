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

class ManageSiteEmailListSubscribersModule extends ManageSiteBaseModule {

	public function build($runData){
		$site = $runData->getTemp('site');
		$pl = $runData->getParameterList();
		$listId = $pl->getParameterValue("listId");

		$db = Database::connection();
		$db->begin();
	
		// get the list		
		$c= new Criteria();
		$c->add('site_id', $site->getSiteId());
		$c->add('list_id', $listId);
		
		$list = DB_EmailListPeer::instance()->selectOne($c);
		
		if(!$list){
			throw new ProcessException('The requested list  cannot be found.');
		}
		
		// get all subscribers
		$q = "SELECT ozone_user.* FROM email_list_subscriber, ozone_user WHERE ".
			"email_list_subscriber.list_id = '{$list->getListId()}' AND email_list_subscriber.user_id = ozone_user.user_id " .
			"ORDER BY ozone_user.nick_name";
			
		$c = new Criteria();
		$c->setExplicitQuery($q);
		
		$users = DB_OzoneUserPeer::instance()->select($c);
		
		$runData->contextAdd('users', $users);
		
		$runData->contextAdd('list',$list);
		$runData->contextAdd('site', $site);

	}
	
}
