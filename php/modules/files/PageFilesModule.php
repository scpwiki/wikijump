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

class PageFilesModule extends SmartyModule{
	
	public function build($runData){
		$site = $runData->getTemp("site");
		
		$pageId = $runData->getParameterList()->getParameterValue("page_id");
		if(!$pageId || !is_numeric($pageId)){
			throw new ProcessException(_("The page can not be found or does not exist."), "no_page");	
		}
		$page = DB_PagePeer::instance()->selectByPrimaryKey($pageId);
		if(!$page || $page->getSiteId() !== $site->getSiteId()){
			throw new ProcessException(_("The page can not be found or does not exist."), "no_page");	
		}
		$files = $page->getFiles();
		
		if(count($files)>0){
			$runData->contextAdd("files", $files);
			$runData->contextAdd("filePath", "/local--files/".$page->getUnixName()."/");
			$totalPageSize = FileHelper::totalPageFilesSize($pageId);
			$totalPageSize = FileHelper::formatSize($totalPageSize);
			$runData->contextAdd("totalPageSize", $totalPageSize);
		}
		
	}
	
}
