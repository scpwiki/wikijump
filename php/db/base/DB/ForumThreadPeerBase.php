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
 * @version \$Id\$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

namespace DB;

use BaseDBPeer;



 
/**
 * Base peer class mapped to the database table forum_thread.
 */
class ForumThreadPeerBase extends BaseDBPeer {
	public static $peerInstance;
	
	protected function internalInit(){
		$this->tableName='forum_thread';
		$this->objectName='DB\\ForumThread';
		$this->primaryKeyName = 'thread_id';
		$this->fieldNames = array( 'thread_id' ,  'user_id' ,  'user_string' ,  'category_id' ,  'title' ,  'description' ,  'number_posts' ,  'date_started' ,  'site_id' ,  'last_post_id' ,  'page_id' ,  'sticky' ,  'blocked' );
		$this->fieldTypes = array( 'thread_id' => 'serial',  'user_id' => 'int',  'user_string' => 'varchar(80)',  'category_id' => 'int',  'title' => 'varchar(256)',  'description' => 'varchar(1000)',  'number_posts' => 'int',  'date_started' => 'timestamp',  'site_id' => 'int',  'last_post_id' => 'int',  'page_id' => 'int',  'sticky' => 'boolean',  'blocked' => 'boolean');
		$this->defaultValues = array( 'number_posts' => '1',  'sticky' => 'false',  'blocked' => 'false');
	}
	
	public static function instance(){
		if(self::$peerInstance == null){
			$className = "DB\\ForumThreadPeer";
			self::$peerInstance = new $className();
		}
		return self::$peerInstance;
	}

}