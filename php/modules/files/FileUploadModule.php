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

class FileUploadModule extends SmartyModule{
	
	public function build($runData){
		$site = $runData->getTemp("site");
		$pl = $runData->getParameterList();
		$pageId = $pl->getParameterValue("pageId");	
		$page = DB_PagePeer::instance()->selectByPrimaryKey($pageId);
		if($page == null || $page->getSiteId() != $site->getSiteId()){
			throw new ProcessException(_("Problem selecting destination page."), "no_page");
		}
		
		$category = $page->getCategory();
		// now check for permissions!!!
		$user = $runData->getUser();
		WDPermissionManager::instance()->hasPagePermission('attach_file', $user, $category, $page);

		$totalSize = FileHelper::totalSiteFilesSize($site->getSiteId());
		$allowed = $site->getSettings()->getFileStorageSize();
		
		$maxUpload = min($allowed - $totalSize, $site->getSettings()->getMaxUploadFileSize());
		
		$runData->contextAdd("totalSiteSize", FileHelper::formatSize($totalSize));
		$runData->contextAdd("totalSiteAllowedSize",  FileHelper::formatSize($allowed));
		$runData->contextAdd("availableSiteSize", FileHelper::formatSize($allowed - $totalSize));
		
		$runData->contextAdd("maxUpload", $maxUpload);
		$runData->contextAdd("maxUploadString",FileHelper::formatSize($maxUpload));
		
	}
	
}
