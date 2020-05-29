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

/**
 * This class handles source changes looking for inslusions and links and
 * tries to change them upon destinatin (or included) page name change.
 */
class DependencyFixer {
	private $page;
	private $oldPageName;
	private $newPageName;
	
	private $user;
	
	public function __construct($page, $oldPageName, $newPageName){
		$this->page = $page;
		$this->oldPageName = $oldPageName;
		$this->newPageName = $newPageName;	
	}
	
	public function fixLinks(){
		// get the current source. for sure check the lock.
		// also note that $page should be selected with "FOR UPDATE" clause
		
		$oldSourceText = $this->page->getSource();
		$sourceChanged = false;
		
		$source = $oldSourceText;
		$source = preg_replace_callback('/(\[\[\[)([^\]\|]+?)((\s*\|[^\]]*?)?\]\]\])/i',  array(&$this, 'fixLink'), $source);
		$source = preg_replace_callback('/^\[\[include ([a-zA-Z0-9\s\-]+?)(?:\]\])$/im', array(&$this, 'fixInclusion'), $source);	
		if($source != $oldSourceText){
			$page = $this->page;
			$currentRevision = $page->getCurrentRevision();
			//save it! wooohaaa! should we not clean the page source saving a bit?
			$revision = clone($currentRevision);
			$revision->setNew(true);
			$revision->setRevisionId(null);
			$revision->resetFlags();
			$revision->setFlagText(true);
			$revision->setPageId($page->getPageId());
			$revision->setRevisionNumber($currentRevision->getRevisionNumber()+1);
			
			$now = new ODate();
			$revision->setDateLastEdited($now);

			$fullSource = false;
			// first check if store new source as a diff or as a full-source.
			if($currentRevision->getSinceFullSource() > 9){
				$fullSource = true;
			} else{
				// also compare size of diff against size of new source.
				// must be less than %50 to qualify
				$differ = new ODiff();
				$diff = $differ->diffString($oldSourceText, $source);
				if(strlen($diff) > 0.5 * strlen($source)){
					$fullSource = true;
				}	
			}
			
			$pageSource = new DB_PageSource();
			if($fullSource){
				$pageSource->setText($source);
				$revision->setDiffSource(false);
				$revision->setSinceFullSource(0);
			}else{
				$pageSource->setText($diff);
				$revision->setDiffSource(true);
				$revision->setSinceFullSource($currentRevision->getSinceFullSource()+1);
			}
			$pageSource->save();
				
			$revision->setSourceId($pageSource->getSourceId());
			
			$revision->setComments(sprintf(_('Automatic update related to page rename: "%s" to "%s".'),$this->oldPageName,$this->newPageName));
		
			$userId = $this->user->getUserId();
		
			if($userId){
				$revision->setUserId($userId);
				$page->setLastEditUserId($userId);
			}

			$revision->save();
			$page->setRevisionId($revision->getRevisionId());
			$page->setDateLastEdited($now);
			$page->setRevisionNumber($revision->getRevisionNumber());
			$page->save();
			
			// force page compilation

		}	
	}
	
	public function fixInclusion($matches){
		$pageName =  WDStringUtils::toUnixName(trim($matches[1]));
		if($pageName != $this->oldPageName){
			return $matches[0];
		}else{
			return	'[[include '.$this->newPageName.']]';
		}
	}
	
	private function fixLink($matches){
		
		$pageName = WDStringUtils::toUnixName($matches[2]);
		$start = $matches[1];
		$rest = $matches[3];
		if($pageName != $this->oldPageName){
			return $matches[0];
		}else{
			return $start.$this->newPageName.$rest;	
		}
	}
	
	public function setUser($user){
		$this->user = $user;	
	}
	
}
