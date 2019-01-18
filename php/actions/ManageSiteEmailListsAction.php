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

class ManageSiteEmailListsAction extends SmartyAction {
	
	public function isAllowed($runData){
		WDPermissionManager::instance()->hasPermission('manage_site', $runData->getUser(), $runData->getTemp("site"));	
		
		return true;
	}
	
	public function perform($r){}
	
	public function saveListEvent($runData){
		
		$pl =  $runData->getParameterList();
		$site = $runData->getTemp("site");
		$siteId = $site->getSiteId();

		$listId = $pl->getParameterValue('listId');
		$isNew = ! (bool) $listId;
		$listTitle = trim($pl->getParameterValue('title'));
		$listUnixName = trim($pl->getParameterValue('unixName'));
		$listWhoCanJoin = trim($pl->getParameterValue('whoCanJoin'));
		
		if(strlen($listTitle) > 30){
			throw new ProcessException('List title can not be longer than 30 characters');
		}
		if(strlen($listTitle) == 0){
			throw new ProcessException('Title of the list should be provided.');
		}
		if(strlen($listUnixName) > 30){
			throw new ProcessException('Unix name (address) of the list can not be longer than 20 characters');
		}
		if(strlen($listUnixName) == 0){
			throw new ProcessException('Unix name (address) of the list should be provided.');
		}
		
		$db = Database::connection();
		$db->begin();
		
		$list = null;
		if($isNew){
			$list = new DB_EmailList();
			$list->setSiteId($siteId);
		}else{
			$c = new Criteria();
			$c->add('list_id', $listId);
			$c->add('site_id', $site->getSiteId());
			$list = DB_EmailListPeer::instance()->selectOne($c);
		}
		$list->setTitle($listTitle);
		$list->setUnixName($listUnixName);
		$list->setWhoCanJoin($listWhoCanJoin);
		
		try{
			$list->save();
		}catch(Excepion $e){
			throw new ProcessExcepion("List cannot be saved.");
		}	
		$db->commit();
		
	}
	
	public function unsubscribeEvent($runData){
		$pl =  $runData->getParameterList();
		$site = $runData->getTemp("site");
		$siteId = $site->getSiteId();
		$listId = $pl->getParameterValue('listId');
		$userId = $pl->getParameterValue('userId');
		$c = new Criteria();
		$c->add('list_id', $listId);
		$c->add('site_id', $site->getSiteId());
		
		$db = Database::connection();
		$db->begin();
		$list = DB_EmailListPeer::instance()->selectOne($c);
		if(!$list){
			throw new ProcessException('The requested list  cannot be found.');
		}
		
		$c = new Criteria();
		$c->add('list_id', $listId);
		$c->add('user_id', $userId);
		
		DB_EmailListSubscriberPeer::instance()->delete($c);
		$list->calculateSubscriberCount();
		$list->save();
		$db->commit();
	}
}
