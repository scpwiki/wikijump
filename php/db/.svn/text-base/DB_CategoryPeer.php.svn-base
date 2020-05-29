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
 * @package Wikidot_Db
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

/**
 * Object Model mapped class.
 *
 */
class DB_CategoryPeer extends DB_CategoryPeerBase {

	public function selectByName($name, $siteId, $useMemcache=true){
		
		if($name !== "_default"){
			$name = WDStringUtils::toUnixName($name);
		}
		
		if($useMemcache){
			$memcache = Ozone::$memcache;
			$key = 'category..'.$siteId.'..'.$name;
			$cat = $memcache->get($key);
			if($cat){
				return $cat;
			} else{ 
			
				$c = new Criteria();
				$c->add("name", $name);
				$c->add("site_id", $siteId);
				$cat = $this->selectOne($c);
				$memcache->set($key, $cat, 0, 864000); // 10 days ;-)	
				return $cat;
			}
		}else{
			$c = new Criteria();
			$c->add("name", $name);
			$c->add("site_id", $siteId);
			$cat = $this->selectOne($c);
			return $cat;
		}
	}
	
	public function selectById($categoryId, $siteId,  $useMemcache=true){

		if($useMemcache){
			$memcache = Ozone::$memcache;
			$key = 'categorybyid..'.$siteId.'..'.$categoryId;
			$cat = $memcache->get($key);
			if($cat != false){
				return $cat;
			} else{ 
			
				$c = new Criteria();
				$c->add("category_id", $categoryId);
				$c->add("site_id", $siteId);
				$cat = $this->selectOne($c);
				$memcache->set($key, $cat, 0, 864000);	
				return $cat;
			}
		}else{
			
			$c = new Criteria();
			$c->add("category_id", $categoryId);
			$c->add("site_id", $siteId);
			$cat = $this->selectOne($c);
			return $cat;
		}
		
	}

}
