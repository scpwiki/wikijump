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

class ManageSiteBackupAction extends SmartyAction {
	
	public function isAllowed($runData){
		WDPermissionManager::instance()->hasPermission('manage_site', $runData->getUser(), $runData->getTemp("site"));	
		return true;
	}
	
	public function perform($r){}
	
	public function requestBackupEvent($runData){
		$pl = $runData->getParameterList();
		
		$backupSources = (bool)$pl->getParameterValue("backupSources");
		$backupFiles = (bool)$pl->getParameterValue("backupFiles");

		if(!$backupSources && !$backupFiles){
			throw new ProcessException(_("So what do you want to backup? Choose the components."));	
		}
		
		$site = $runData->getTemp("site");
		
		$c = new Criteria();
		$c->add("site_id", $site->getSiteId());
		DB_SiteBackupPeer::instance()->delete($c);
		
		$sb = new DB_SiteBackup();
		$sb->setSiteId($site->getSiteId());
		$sb->setBackupSource($backupSources);
		$sb->setBackupFiles($backupFiles);
		$sb->setDate(new ODate());
		
		$sb->save();
	}
	
	public function deleteBackupEvent($runData){
		$site = $runData->getTemp("site");
		
		$c = new Criteria();
		$c->add("site_id", $site->getSiteId());
		DB_SiteBackupPeer::instance()->delete($c);
		
		@exec('rm -r '.WIKIDOT_ROOT.'/web/files--sites/'.$site->getUnixName().'/backup');	
	}
	
}
