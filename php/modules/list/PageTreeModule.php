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

class PageTreeModule extends SmartyModule {
	
	public function build($runData){
		
		$pl = $runData->getParameterList();
		$site = $runData->getTemp("site");
		
		$root = $pl->getParameterValue("root");
		$showRoot = $pl->getParameterValue("showRoot");
		if(!$root){
			$page = $runData->getTemp("page");
		}else{
		
			$page = DB_PagePeer::instance()->selectByName($site->getSiteId(), $root);
		}
		if(!$page){
			$runData->setModuleTemplate("Empty");
			return;	
		}
		
		$depth = $pl->getParameterValue("depth");
		if(!$depth || !is_numeric($depth) || $depth<1){
			$depth = 5;	
		}
		
		$tree = array();
		
		$c = new Criteria();
		$c->add("parent_page_id", $page->getPageId());
		$c->addOrderAscending("COALESCE(title, unix_name)");
		$children = DB_PagePeer::instance()->select($c);
		
		$descendants = array();
		// select next level of children
		$ch1 = $children;
		$d = 0;
		while($ch1 != null && count($ch1)>0 && (!$depth || $d<$depth)){
			$q = "SELECT * FROM page WHERE parent_page_id IN (";
			$tch = array();
			foreach($ch1 as $ch){
				// check if already in the $tch
				if(!array_key_exists($ch->getPageId(), $descendants)){
					$tch[] = $ch->getPageId();
					$descendants[$ch->getParentPageId()][] = $ch;
				}else{
					$ch->setTemp('circular', true);
					$descendants[$ch->getParentPageId()][] = $ch;
				}
				
			}
			if(count($tch)>0){
				$q .= implode(',', $tch);
				$q .= ") ORDER BY COALESCE(title, unix_name)";
				$c = new Criteria();
				$c->setExplicitQuery($q);
				$ch2 = DB_PagePeer::instance()->select($c);
				$ch1 = $ch2;
			}else{
				$ch1 = null;
			}
			$d++;
		}
			
		$runData->contextAdd("root", $page);
		$runData->contextAdd("children", $descendants);
		
		if($showRoot){
			$runData->contextAdd("showRoot", true);
		}

	}
	
	/*
	private function naturalSort(&$pages){
		$ref = array();
		$sor = array();
		foreach($pages as $key => $page){
			$name = $page->getTitleOrUnixName();
			$ref[$name] = $key;
			$sor[] = $name;	
		}
		natsort($sor);
		$out = array();
		foreach($sor as $name);
		$out[] = $pages[$ref[$name]];
		$pages = $out;
	}
	*/
}
