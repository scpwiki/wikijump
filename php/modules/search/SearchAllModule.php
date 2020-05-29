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

class SearchAllModule extends SmartyModule {

	protected function normalizeWhiteSpace($query) {
		return trim(preg_replace('/\s+/', ' ', $query));
	}
	
	protected function parseQuery($query) {
		// add some space
		$q = " $query ";
		
		// check for site:X,Y,Z strings
		$sites = null;
		$m = array();
		if (preg_match("/ site:([a-z0-9,-]+) /i", $q, $m)) {
			$sites = explode(",", $m[1]);
			$q = preg_replace("/ site:([a-z0-9,-]+) /i", "", $q);
		}
		
		// we want "pure" query version now
		// escaping \, !, (, ), :, ^, [, ], {, }, ~, *, ?
		$q = preg_replace('/[&\|\?~,)("^!{}[]/', " ", $q);
		$q = str_replace(']', " ", $q);
		$q = preg_replace('/([a-z][a-z][a-z])\*/', '\1~', $q);
		$q = str_replace('*', ' ', $q);
		$q = str_replace("~", '*', $q);
		$q = str_replace("tags:", "tags~", $q);
		$q = str_replace("tag:", "tags~", $q);
		$q = str_replace("title:", "title~", $q);
		$q = str_replace("content:", "content~", $q);
		$q = str_replace(":", " ", $q);
		$q = str_replace("~", ":", $q);
		
		$q = $this->normalizeWhiteSpace($q);
		
		return array("sites" => $sites, "query" => $q);
	}
	
	protected function simplifyForTs($query) {
		$q = " $query ";
		$q = preg_replace("/ site:[a-z0-9,-]+/i", " ", $q);
		$q = $this->normalizeWhiteSpace($q);
		$q = preg_replace("/[&\|:\?^~]/", ' ', $q);
		$q = preg_replace("/((^)|([\s]+))\-/", '&!', $q);
		$q = str_replace("-", " ", $q);
		$q = trim($q);
		$q = preg_replace('/ +/', '&', $q);
		return $q;
	}
	
	public function build($runData){
		
		// parse parameters
		$pl = $runData->getParameterList();
		$query = trim($pl->getParameterValue("q"));
		$area = $pl->getParameterValue("a");
		
		if($query == ''){
			return;	
		}
		if(strlen($query)<3){
			$runData->contextAdd("query", $query);
			$runData->contextAdd("errorMessage", _("Your query should be at least 3 characters long.")); 
			return;	
		}
		
		// pagination
		$pageNumber = $pl->getParameterValue("p");
		if($pageNumber == null || !is_numeric($pageNumber) || $pageNumber <1){
			$pageNumber = 1;	
		}
		$perPage = 10;
		$limit = $perPage;
		$offset = ($pageNumber - 1) * $perPage;
		
		// parse query
		$query_array = $this->parseQuery($query);
		$ts_query = "'" . db_escape_string($this->simplifyForTs($query_array['query'])) . "'";
		
		// find
		$lucene = new Wikidot_Search_Lucene();
		$lucene_hits = $lucene->search($query_array['query'], $runData->getUser(), $area, $query_array['sites']);
		$result_count = count($lucene_hits);
		
		// limit
		$lucene_hits = array_slice($lucene_hits, $offset, $limit);
		
		// hedline options
		$headlineOptions = "'MaxWords=200, MinWords=100'";
		
		// fetch items from database with highlight
		$db = Database::connection();
    	$v = pg_version($db->getLink());
    	
		if (!preg_match(';^8\.3;', $v['server'])) {
		    $db->query("SELECT set_curcfg('default')");
		} else {
			$tsprefix = 'ts_'; // because in postgresql 8.3 functions are ts_rank and ts_header
		}
		
		$res = array();
		
		foreach ($lucene_hits as $fts_id) {
			
			$q = "SELECT *, 
					fts_entry.unix_name AS fts_unix_name, 
					{$tsprefix}headline(text, q, 'MaxWords=50, MinWords=30') AS headline_text, 
					{$tsprefix}headline(title, q, $headlineOptions) AS headline_title 
				FROM fts_entry, site, to_tsquery($ts_query) AS q
				WHERE fts_id = $fts_id AND fts_entry.site_id = site.site_id";
			
			file_put_contents("/tmp/debug-query", "$q\n");
			
			$r = $db->query($q);
			$res_one = $r->fetchAll();
			
			if ($res_one && count($res_one)) {
				$res[] = $res_one[0];
			}
		}
		
		// pager data
		$total_pages = ceil($result_count / $perPage);
		$pagerData = array();
		$pagerData['current_page'] = $pageNumber;
		$pagerData['known_pages'] = min(array($pageNumber + 2, $total_pages));
		$pagerData['total_pages'] = $total_pages;
		
		// construct URLs
		for ($i = 0; $i < count($res); $i++) {
			$o = $res[$i];
			$res[$i]['site'] = new DB_Site($res[$i]);
			if($o['page_id'] !== null){
				$res[$i]['url'] = 'http://'.$res[$i]['site']->getDomain().'/'.$o['fts_unix_name'];	
			}else{
				$res[$i]['url'] = 'http://'.$res[$i]['site']->getDomain().'/forum/t-'.$o['thread_id'].'/'.$o['unix_name'];	
			}
		}
		
		// feed the template
		$runData->contextAdd("pagerData", $pagerData);
		$runData->contextAdd("results", $res);
		$runData->contextAdd("countResults", count($res));
		$runData->contextAdd("totalResults", $result_count);
		$runData->contextAdd("query", $query);
		$runData->contextAdd("encodedQuery", urldecode($query));
		$runData->contextAdd("queryEncoded", urlencode($query));
		$runData->contextAdd("area", $area);
		//$runData->contextAdd("query_debug", $lucene_query); 
		$runData->contextAdd("domain", $runData->getTemp("site")->getDomain());
		
	}
	
}
