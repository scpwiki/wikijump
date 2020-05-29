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

class PagePreviewModule extends SmartyModule {
	
	public function build($runData){
		$pl = $runData->getParameterList();
		$source = $pl->getParameterValue("source");
		$mode = $pl->getParameterValue("mode");
	
		$site = $runData->getTemp("site");
	
		$pageId = $pl->getParameterValue("pageId");
		if($pageId){
			$runData->setTemp("pageId", $pageId);	
			$page = DB_PagePeer::instance()->selectByPrimaryKey($pageId);
			if($page == null || $page->getSiteId() != $site->getSiteId()){
				throw new ProcessException(_("Error selecting the page."));	
			}
			$runData->setTemp("page", $page);
		}

		$wt = new WikiTransformation();
		$wt->setPageUnixName($pl->getParameterValue("page_unix_name"));
		
	/*	if($mode == "append"){
			// combine current source and submitted source
			$pageId = $pl->getParameterValue("page_id");
			$page = DB_PagePeer::instance()->selectByPrimaryKey($pageId);
			$source = $page->getSource()."\n\n[[div id=\"append-preview-div\"]]\n".$source."\n[[/div]]\n";
		}
		*/
		
		/* Get the category and apply the "live template" to the source. */
		$pageUnixName = $pl->getParameterValue("page_unix_name");
		
	    if(strpos( $pageUnixName, ":") != false){
			$tmp0 = explode(':',$pageUnixName); 
			$categoryName = $tmp0[0];
		} else {
			$categoryName = "_default";
		}

		$category = DB_CategoryPeer::instance()->selectByName($categoryName, $site->getSiteId());
		
		/* Look for the template (if any). */
		if(!preg_match(';(:|^)_;', $pageUnixName)) {
		$templatePage = DB_PagePeer::instance()->selectByName($site->getSiteId(), 
		    ($categoryName == '_default' ? '' : $categoryName.':') .'_template');
		
		if($templatePage) {
    	    $source = $wt->assemblyTemplate($source, $templatePage->getSource());
		}
		}
	    
		$result = $wt->processSource($source);
		
		$body = $result;
		$runData->contextAdd("body", $body);
		$runData->ajaxResponseAdd("title", $pl->getParameterValue("title"));
		
	}
	
}
