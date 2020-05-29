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

class PageDiffModule extends SmartyModule {
	
	public function build($runData){
		$pl = $runData->getParameterList();
		
		$fromRevisionId = $pl->getParameterValue("from_revision_id");
		$toRevisionId  = $pl->getParameterValue("to_revision_id");
		
		if($fromRevisionId == $toRevisionId){
			throw new ProcessException(_("What is the point in comparing the revision with itself? Please choose different revisions of the page."), "same_revision");	
		}

		$fromRevision = DB_PageRevisionPeer::instance()->selectByPrimaryKey($fromRevisionId);
		$toRevision = DB_PageRevisionPeer::instance()->selectByPrimaryKey($toRevisionId);
		
		if($fromRevision == null || $toRevision == null){
			throw new ProcessException(_("Error selecting revisions to compare"), "no_revisions");
		}
		
		$fromMetadata = $fromRevision->getMetadata();
		$toMetadata = $toRevision->getMetadata();
		
		$changed = array();
		
		// compare titles and other things
		if($fromMetadata->getTitle() !== $toMetadata->getTitle()){
			$changed['title'] = true;
		}
		if($fromMetadata->getUnixName() !== $toMetadata->getUnixName()){
			$changed['unix_name'] = true;
		}
		if($fromMetadata->getParentPageId() !== $toMetadata->getParentPageId()){
			$changed['parent'] = true;
			if($fromMetadata->getParentPageId()){
				$fromParent = DB_PagePeer::instance()->selectByPrimaryKey($fromMetadata->getParentPageId())->getUnixName();
				$runData->contextAdd("fromParent", $fromParent);
			}
			if($toMetadata->getParentPageId()){
				$toParent = DB_PagePeer::instance()->selectByPrimaryKey($toMetadata->getParentPageId())->getUnixName();
				$runData->contextAdd("toParent", $toParent);
			}
			
		}
		
		//compare source now
		
		$fromPageSource = $fromRevision->getSourceText();
		$toPageSource = $toRevision->getSourceText();
		
		if($fromPageSource !== $toPageSource){
			$changed['source'] = true;
		
			// create page diff... wooo...
			
			$t1 = $fromPageSource;
			$t2 = $toPageSource;

			$inlineDiff = Wikidot_Util_Diff::generateInlineStringDiff($t1, $t2);
			$runData->contextAdd("inlineDiff", $inlineDiff	);
			
		}
		$runData->contextAdd("fromPageSource", $fromPageSource);
		$runData->contextAdd("toPageSource", $toPageSource);
		
		$runData->contextAdd("fromRevision", $fromRevision);
		$runData->contextAdd("toRevision", $toRevision);
		$runData->contextAdd("fromMetadata", $fromMetadata);
		$runData->contextAdd("toMetadata", $toMetadata);
		
		$runData->contextAdd("changed", $changed);

	}
	
}
