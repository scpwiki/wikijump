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
 * @category Ozone
 * @package Ozone_Web
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

/**
 * Simple db-based screen cache manager.
 *
 */
class ScreenCacheManager {
	
	public static $manager;
	
	public static function instance(){
		if(self::$manager == null){
			self::$manager = new ScreenCacheManager();	
		}	
		return self::$manager;
	}
	
	public function cachedLayout($runData, $screenCacheSettings){
		$c = new Criteria();
		$c->add("template", $runData->getScreenTemplate());
		$c->add("request_uri", $runData->getRequestUri());
		$c->add("type", "layout");
		$c->add("user_authenticated", $runData->isUserAuthenticated());	
		
		$timeout = $screenCacheSettings->getLayoutTimeout($runData);
		// it was in seconds. make date with maximum time allowed
		$date = new ODate();
		$date->subtractSeconds($timeout);
		$c->add("date_updated", $date, ">");
		
		$sc = DB_ScreenCachePeer::instance()->selectOne($c);
		if($sc != null){
			return $sc->getContent();	
		}
		return null;
	}

	public function cachedScreen($runData, $screenCacheSettings){
		$c = new Criteria();
		$c->add("template", $runData->getScreenTemplate());
		$c->add("request_uri", $runData->getRequestUri());
		$c->add("type", "screen");
		$c->add("user_authenticated", $runData->isUserAuthenticated());	
		
		$timeout = $screenCacheSettings->getScreenTimeout($runData);
		// it was in seconds. make date with maximum time allowed
		$date = new ODate();
		$date->subtractSeconds($timeout);
		$c->add("date_updated", $date, ">");
		
		$sc = DB_ScreenCachePeer::instance()->selectOne($c);
		if($sc != null){
			return $sc->getContent();	
		}
		return null;
	}
	
	public function updateCachedLayout($runData, $content){
		// delete any previous cache content for this request
		$this->deleteCachedLayout($runData);
		
		$sc = new DB_ScreenCache();
		$sc->setTemplate($runData->getScreenTemplate());
		$sc->setDateUpdated(new ODate());
		$sc->setType("layout");
		$sc->setUserAuthenticated($runData->isUserAuthenticated());
		$sc->setRequestUri($runData->getRequestUri());
		$sc->setContent($content);
		
		$sc->save();

	}
	
	public function updateCachedScreen($runData, $content){
		// delete any previous cache content for this request
		$this->deleteCachedScreen($runData);
		
		$sc = new DB_ScreenCache();
		$sc->setTemplate($runData->getScreenTemplate());
		$sc->setDateUpdated(new ODate());
		$sc->setType("screen");
		$sc->setUserAuthenticated($runData->isUserAuthenticated());
		$sc->setRequestUri($runData->getRequestUri());
		$sc->setContent($content);
		
		echo $content;
		
		$sc->save();	
	}
	
	public function deleteCachedLayout($runData){
		$c = new Criteria();
		$c->add("template", $runData->getScreenTemplate());
		$c->add("request_uri", $runData->getRequestUri());
		$c->add("type", "layout");
		$c->add("user_authenticated", $runData->isUserAuthenticated());	
		
		DB_ScreenCachePeer::instance()->delete($c);
	}
	
	public function deleteCachedScreen($runData){
		$c = new Criteria();
		$c->add("template", $runData->getScreenTemplate());
		$c->add("request_uri", $runData->getRequestUri());
		$c->add("type", "screen");
		$c->add("user_authenticated", $runData->isUserAuthenticated());	
		
		DB_ScreenCachePeer::instance()->delete($c);
	}
	
	public function clearCache($template=null){
			
	}
}
