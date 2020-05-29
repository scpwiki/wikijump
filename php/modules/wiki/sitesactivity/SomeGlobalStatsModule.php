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

class SomeGlobalStatsModule extends SmartyModule {
	
	protected $timeOut=3600;
	
	public function render($runData){
		$pl = $runData->getParameterList();
		
		$site = $runData->getTemp("site");
		$range = $pl->getParameterValue("range", "AMODULE");
		
		$key = "module..0..SomeGlobalStatsModule";
		$mc = OZONE::$memcache;
		
		$out = $mc->get($key);
		if(!$out){
			$out = parent::render($runData);
			$mc->set($key, $out, 0, 600);	
		} 
		
		return $out;
		
	}
	
	public function build($runData){
		// just get some numbers
		
		$db = Database::connection();
		
		$q = "SELECT count(*) AS c FROM ozone_user";
		$res = $db->query($q);
		$row = $res->nextRow();
		$totalUsers = $row['c'];
		/* -2 because there are 2 "ghost users" in the default installation 
			with user_id < 0 */
		$runData->contextAdd("totalUsers", $totalUsers - 2);
		
		$q = "SELECT count(*) AS c FROM site";
		$res = $db->query($q);
		$row = $res->nextRow();
		$totalSites = $row['c'];
		$runData->contextAdd("totalSites", $totalSites);
		
		$q = "SELECT count(*) AS c FROM page";
		$res = $db->query($q);
		$row = $res->nextRow();
		$totalPages = $row['c'];
		$runData->contextAdd("totalPages", $totalPages);
		
		$q = "SELECT count(*) AS c FROM ozone_user WHERE registered_date>now() - interval '1 day'";
		$res = $db->query($q);
		$row = $res->nextRow();
		$newUsers = $row['c'];
		$runData->contextAdd("newUsers", $newUsers);
		
		$q = "SELECT count(*) AS c FROM page_revision WHERE date_last_edited>now() - interval '1 day'";
		$res = $db->query($q);
		$row = $res->nextRow();
		$recentEdits = $row['c'];
		$runData->contextAdd("recentEdits", $recentEdits);
		
	}

}
