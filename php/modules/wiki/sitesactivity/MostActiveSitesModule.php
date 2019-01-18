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

class MostActiveSitesModule extends SmartyModule {
	
	protected $timeOut=3600;
	
	public function render($runData){
		$pl = $runData->getParameterList();
		$site = $runData->getTemp("site");
		
		$range = $pl->getParameterValue("range", "AMODULE");
		
		$key = "module..0..MostActiveSitesModule..".$site->getSiteId().'..'.$range;
		$mc = OZONE::$memcache;
		
		$out = $mc->get($key);
		if(!$out){
			$out = parent::render($runData);
			$mc->set($key, $out, 0, 3600);	
		} 
		
		return $out;
		
	}
	
	public  function build($runData){
		
		$pl = $runData->getParameterList();
		
		$range = $pl->getParameterValue("range", "AMODULE");
		$dateStart = new ODate();
		
		if(!in_array($range, array('24h', '7days', 'month'))){
			$range = '7days';
		}
		
		switch($range){
			case '24h':
				$dateStart->addSeconds(-60*60*24);
				break;
			case '7days':
				$dateStart->addSeconds(-60*60*24*7);
				break;
			case 'month':
				$dateStart->addSeconds(-60*60*24*31);
				break;	
		}	
		$q = "SELECT site.site_id, count(*) AS number_changes FROM site, page_revision WHERE page_revision.date_last_edited > '".$dateStart->getDate()."' AND site.visible = TRUE AND site.private = FALSE AND site.deleted = FALSE AND site.site_id != 1 AND page_revision.flag_new_site=FALSE AND page_revision.site_id = site.site_id GROUP BY site.site_id ORDER BY number_changes DESC LIMIT 10";
		
		$db = Database::connection();
		
		$res = $db->query($q);
		
		$all = $res->fetchAll();
		if($all){
			foreach($all as &$a){
				$a['site'] = DB_SitePeer::instance()->selectByPrimaryKey($a['site_id']);
			}
		}
		
		$runData->contextAdd("res", $all);
		$runData->contextAdd("range", $range);
			
	}
	
}
