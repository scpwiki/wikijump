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
	
	public function build($runData){

		$pl = $runData->getParameterList();
		$query = trim($pl->getParameterValue("q"));
		$area = $pl->getParameterValue("a");
		
		if($area != 'p' && $area != 'f' && $area != 'pf'){
			$area = null;
		}
		
		if($query == ''){
			return;	
		}
		if(strlen($query)<3){
			$runData->contextAdd("query", $query);
			$runData->contextAdd("errorMessage", _("Your query should be at least 3 characters long.")); 
			return;	
		}
		
		$site = $runData->getTemp("site");
		
		// pagination
		
		$pageNumber = $pl->getParameterValue("p");
		if($pageNumber == null || !is_numeric($pageNumber) || $pageNumber <1){
			$pageNumber = 1;	
		}
		$perPage = 10;
		
		$limit = $perPage*2+1;
		$offset = ($pageNumber - 1)*$perPage;
		
		$qe = $query;
		$qe = preg_replace("/[!:\?]/",' ', $qe);
		$qe = preg_replace("/[&\|!]+/", ' ', $qe);
		$qe = preg_replace("/((^)|([\s]+))\-/", '&!', $qe);
		$qe = str_replace("-", " ", $qe);
		$qe = trim($qe);
		$qe = preg_replace('/ +/', '&', $qe);
		// prepare fts query
		
		// escaped query
		$eq = "'".db_escape_string($qe)."'";
		
		// search pages
		$headlineOptions = "'MaxWords=200, MinWords=100'";
		
		$db = Database::connection();

    	$v = pg_version($db->getLink());
    	
		if(!preg_match(';^8\.3;', $v['server'])){
		    $db->query("SELECT set_curcfg('default')");
		} else {
			$tsprefix = 'ts_'; // because in postgresql 8.3 functions are ts_rank and ts_header
		}
		
		$q = "SELECT *, fts_entry.unix_name AS fts_unix_name, {$tsprefix}headline(text, q, 'MaxWords=50, MinWords=30') AS headline_text, {$tsprefix}headline(title, q, $headlineOptions) AS headline_title FROM fts_entry, site, to_tsquery($eq) AS q " .
			"WHERE site.visible=TRUE AND site.private = FALSE AND site.deleted = FALSE";
	
		
		if($area){
			
			switch($area){
				case 'f':
					$q .= " AND thread_id IS NOT NULL ";
					break;
				case 'p':
					$q .= " AND page_id IS NOT NULL ";
					break;
			}
				
		}
		$q .= " AND " .
				"vector @@ q " .
				"AND fts_entry.site_id=site.site_id " .
				"ORDER BY {$tsprefix}rank(vector, q) DESC LIMIT $limit OFFSET $offset";
		
		$r = $db->query($q);
		$res = $r->fetchAll();

		if($res){
			// fix urls
			$counted = count($res); 
		
			$pagerData = array();
			$pagerData['current_page'] = $pageNumber;
			if($counted >$perPage*2){
				$knownPages=$pageNumber + 2;
				$pagerData['known_pages'] = $knownPages;	
			} elseif($counted>$perPage){
				$knownPages=$pageNumber + 1;
				$pagerData['total_pages'] = $knownPages; 
			}else{
				$totalPages = $pageNumber;	
				$pagerData['total_pages'] = $totalPages;
			}
			
			$res = array_slice($res, 0, $perPage);
			for($i=0; $i<count($res); $i++){
				$o = $res[$i];
				$res[$i]['site'] = new DB_Site($res[$i]);
				if($o['page_id'] !== null){
					$res[$i]['url'] = 'http://'.$res[$i]['site']->getDomain().'/'.$o['fts_unix_name'];	
				}else{
					$res[$i]['url'] = 'http://'.$res[$i]['site']->getDomain().'/forum/t-'.$o['thread_id'].'/'.$o['unix_name'];	
				}
			}
			
		}
		
		$runData->contextAdd("pagerData", $pagerData);

		$runData->contextAdd("results", $res);
		$runData->contextAdd("countResults", count($res));
		$runData->contextAdd("query", $query);
		$runData->contextAdd("encodedQuery", urldecode($query));
		$runData->contextAdd("queryEncoded", urlencode($query));
		$runData->contextAdd("area", $area);
		$runData->contextAdd("query_debug", $qe); 
		$runData->contextAdd("domain", $site->getDomain());
		
	}
	
}
