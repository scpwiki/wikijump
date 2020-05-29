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

/*
 * Wikidot version 1
 * 
 * Copyright 2008 by Wikidot Inc.
 * 
 * This program is free software available under the 
 * GNU AFFERO GENERAL PUBLIC LICENSE version 3.
 */
require(WIKIDOT_ROOT."/lib/phpFlickr/phpFlickr.php");

class FlickrHandler extends phpFlickr {
	
	 public $cache = true;
	
	private static $instance;
	
	public static function instance(){
		if(self::$instance == null){
			// get the flickr key
			$key = file_get_contents(WIKIDOT_ROOT.'/files/flickr-api-key.txt');	
			self::$instance = new FlickrHandler($key, null, false);	
		}
		return self::$instance;
	}	 
	
	 function enableCache($type, $connection, $cache_expire = 600, $table = 'flickr_cache'){}
	 
	 function getCached ($request){
	 	$reqhash = md5(serialize($request));
	 	$key = "phpflickrcache..".$reqhash;
	 	$mc = Ozone::$memcache;
		$out = $mc->get($key);
		if($out != false){
			return $out;
		}
	 	return false; 
	 }
	 
	 public function cache ($request, $response){
	 	$reqhash = md5(serialize($request));
	 	$key = "phpflickrcache..".$reqhash;	
	 	$mc = Ozone::$memcache;
	 	$mc->set($key, $response, 0, 600);
	 	return false;
	 }
}
