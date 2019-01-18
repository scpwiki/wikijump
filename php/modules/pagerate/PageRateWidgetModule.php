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

class PageRateWidgetModule extends SmartyModule {
	
	public function build($runData){
		$page = $runData->getTemp("page");
		if($page){
			$rate = $page->getRate();
		}else{
			$pl = $runData->getParameterList();
			$pageId = $pl->getParameterValue("pageId");
			if($pageId){
				$page = DB_PagePeer::instance()->selectByPrimaryKey($pageId);
				$rate = $page->getRate();
			}else{
				$rate = 0;
			}
		}
		
		// get the category too
		if(!$page){
			$site = $runData->getTemp("site");
			$category = DB_CategoryPeer::instance()->selectByName('_default', $site->getSiteId());	
		}else{
			$category = $runData->getTemp("category");
			if(!$category){
				$category = DB_CategoryPeer::instance()->selectByPrimaryKey($page->getCategoryId());
			}
		}
		$type = $category->getRatingType();
		$runData->contextAdd("type", $type);
		$runData->contextAdd("rate", $rate);	
	}
	
}
