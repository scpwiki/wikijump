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

class ViewSourceModule extends SmartyModule{
	public function build($runData){
		$site = $runData->getTemp("site");
		$pageId = $runData->getParameterList()->getParameterValue("page_id");
		
		$raw = $runData->getParameterList()->getParameterValue("raw");
		
		if(!$pageId || !is_numeric($pageId)){
			throw new ProcessException(_("The page can not be found or does not exist."), "no_page");	
		}
		
		$page = DB_PagePeer::instance()->selectByPrimaryKey($pageId);
		
		if(!$page || $page->getSiteId() !== $site->getSiteId()){
			throw new ProcessException(_("The page can not be found or does not exist."), "no_page");	
		}
		
		$source = $page->getCurrentRevision()->getSourceText();
		
		$runData->contextAdd("source", $source);	
		$runData->contextAdd("raw", $raw);	
	}
	
}
