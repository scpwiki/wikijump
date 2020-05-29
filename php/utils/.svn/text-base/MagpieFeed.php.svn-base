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

define("MAGPIE_DIR", WIKIDOT_ROOT."/lib/magpierss/");
define("MAGPIE_CACHE_ON", false);
define("MAGPIE_OUTPUT_ENCODING", "UTF-8");

require(WIKIDOT_ROOT."/lib/magpierss/rss_fetch.inc");

class MagpieFeed {
	
	public function fetch($url){

		// check if not already in cache (memcache only)
		$mc = Ozone::$memcache;
		
		$key = "feed..".$url;
		$o = $mc->get($key);
		if($o != false && $o != null){
			return $o;	
		}
		
		// not in cache, proceed!!!
		
		$o = fetch_rss($url);
		$mc->set($key, $o, 0, 300);
		
		return $o;	
	} 

	public static function getUnixTimestamp($item) {
 		$rss_2_date = $item['pubdate'];
 		$rss_1_date = $item['dc']['date'];
 		$atom_date  = $item['issued'];
 
 		if ($atom_date != "")  $date = parse_w3cdtf($atom_date);
 		if ($rss_1_date != "") $date = parse_w3cdtf($rss_1_date);
 		if ($rss_2_date != "") $date = strtotime($rss_2_date);
 
 		return $date;
	}

}
