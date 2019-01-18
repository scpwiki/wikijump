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

class UserSearchModule extends SmartyModule {
	
	public function build($runData){
		$pl = $runData->getParameterList();
		$query = trim($pl->getParameterValue("q"));

		$site = $runData->getTemp("site");
		
		if(!$query || $query == ''){
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

		// determine the mode: by email or by screenname/realname/unixname
		
		if(strpos($query, '@')){
			// email lookup mode
			$c = new Criteria();
			$c->add("ozone_user.name", $query);
			$user = DB_OzoneUserPeer::instance()->selectOne($c);
			$runData->contextAdd("user", $user);
			$runData->contextAdd("mode", "email");	
			sleep(2);
		}else{
			
			// normal search. perform a regexp search at the moment.
			$qs = preg_split('/ +/', trim($query));
			
			$c = new Criteria();
			foreach($qs as $q){
				$csub = new Criteria();
				$csub->add("ozone_user.nick_name", preg_quote($q), '~*');
				$csub->addOr("ozone_user.unix_name", preg_quote($q), '~*');
				$csub->addOr("profile.real_name", preg_quote($q), '~*');
					
				$c->addCriteriaAnd($csub);
			}

			$c->addJoin("user_id", "profile.user_id");
		
			$limit = $perPage*2+1;
			$offset = ($pageNumber - 1)*$perPage;

			$c->setLimit($limit, $offset);
		
			$res = DB_OzoneUserPeer::instance()->select($c);		
			
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
			
		}
		
		$runData->contextAdd("pagerData", $pagerData);

		$runData->contextAdd("users", $res);
		$runData->contextAdd("countResults", count($res));
		$runData->contextAdd("query", $query);
		$runData->contextAdd("encodedQuery", urldecode($query));
		$runData->contextAdd("queryEncoded", urlencode($query));

	}
	
}
