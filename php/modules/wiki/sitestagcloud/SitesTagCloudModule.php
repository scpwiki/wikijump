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

class SitesTagCloudModule extends CacheableModule {
	
	protected $timeOut=300;
	
	public function build($runData){
		
		$pl = $runData->getParameterList();
		$lang = $pl->getParameterValue("lang");
		
		if($lang && $lang !== "pl" && $lang !== "en"){
			$lang = null;
		}
		
		$target = "/sites-by-tags/tag/";
		if($pl->getParameterValue('target')){
		    $target = '/'.$pl->getParameterValue('target').'/tag/';
		}
		
		$db = Database::connection();
		//select tags
		
		$limit = $pl->getParameterValue("limit");
		
		if($limit && is_numeric($limit) && $limit>0){
				
		}else{
			$limit = 60;	
		}
		
		if($lang){
			$ql = " AND site.language = '".db_escape_string($lang)."' ";		
		}
		
		$q = 'SELECT * FROM (SELECT tag, COUNT(*) AS weight FROM site_tag, site WHERE site.visible = TRUE AND site.private = FALSE AND site.deleted=FALSE '.$ql.' AND site.site_id = site_tag.site_id GROUP BY tag ORDER BY weight DESC LIMIT 100) AS foo ORDER BY tag';
		
		$res = $db->query($q);
		$tags = $res->fetchAll();

		$colorSmall = array(128,128,192);
		$colorBig = array(64,64,128);
		
		$sizeSmall = 25; // percent
		$sizeBig = 100;	// percent
		
		$minWeight = 10000000;
		$maxWeight = 0;
		
		if(!$tags){
			return;
		}
		
		foreach($tags as $tag){
			if($tag['weight'] > $maxWeight){
				$maxWeight = $tag['weight'];
			}
			if($tag['weight'] < $minWeight){
				$minWeight = $tag['weight'];
			}
		}

		$weightRange = $maxWeight - $minWeight;
		
		// now set color and font size for each of the tags.
		
		foreach($tags as &$tag){
			if($weightRange == 0){
				$a = 0;
			}else{
				$a = ($tag['weight']-$minWeight)/$weightRange;
			}
			
			$fontSize = round($sizeSmall + ($sizeBig-$sizeSmall)*$a);
			
			// hadle colors... woooo! excited!
			
			$color = array();
			$color['r'] = round($colorSmall[0] + ($colorBig[0] - $colorSmall[0])*$a);
			$color['g'] = round($colorSmall[1] + ($colorBig[1] - $colorSmall[1])*$a);
			$color['b'] = round($colorSmall[2] + ($colorBig[2] - $colorSmall[2])*$a);
			
			$tag['size'] = $fontSize;
			$tag['color'] = $color; 
		}

		$runData->contextAdd("tags", $tags);
		$runData->contextAdd("href", $target);
		
	}
}
