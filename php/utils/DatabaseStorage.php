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

class DatabaseStorage {
	
	private static $instance;
	
	public static function instance(){
		if(!self::$instance){
			self::$instance = new DatabaseStorage();	
		}
		return self::$instance;
			
	}
	
	public function set($key, $value, $timeout){

		// delete it if already in the database
		DB_StorageItemPeer::instance()->deleteByPrimaryKey($key);
		if(!$value){
			return;	
		}
		$item = new DB_StorageItem();
		$item->setItemId($key);
		$item->setData($value);
		$item->setTimeout($timeout);
		$item->setDate(new ODate());
			
		$item->save();
	}
	
	public function get($key){
		$item = DB_StorageItemPeer::instance()->selectByPrimaryKey($key);
		if($item){
			$timestamp = $item->getDate()->getTimestamp() + $item->getTimeout();

			if($timestamp < time()){
				
				// delete the item, it is outdated!	
				DB_StorageItemPeer::instance()->deleteByPrimaryKey($key);
			}else{
				
				return $item->getData();	
			}	
		}
		return null;	
	}
	
	/**
	 * Cleans outdated items from the database.
	 */
	public function clean(){
		$date = new ODate();
		$c = new Criteria();
		$c->add("date + (timeout || 'sec')::interval", new ODate(), '<');
		DB_StorageItemPeer::instance()->delete($c);
	}
	
}
