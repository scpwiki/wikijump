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
 * Base peer class mapped to the database table forum_post.
 */
class ForumPostPeerBase extends BaseDBPeer {
	public static $peerInstance;
	
	protected function internalInit(){
		$this->tableName='forum_post';
		$this->objectName='DB_ForumPost';
		$this->primaryKeyName = 'post_id';
		$this->fieldNames = array( 'post_id' ,  'thread_id' ,  'parent_id' ,  'user_id' ,  'user_string' ,  'title' ,  'text' ,  'date_posted' ,  'site_id' ,  'revision_number' ,  'revision_id' ,  'date_last_edited' ,  'edited_user_id' ,  'edited_user_string' );
		$this->fieldTypes = array( 'post_id' => 'serial',  'thread_id' => 'int',  'parent_id' => 'int',  'user_id' => 'int',  'user_string' => 'varchar(80)',  'title' => 'varchar(256)',  'text' => 'text',  'date_posted' => 'timestamp',  'site_id' => 'int',  'revision_number' => 'int',  'revision_id' => 'int',  'date_last_edited' => 'timestamp',  'edited_user_id' => 'int',  'edited_user_string' => 'varchar(80)');
		$this->defaultValues = array( 'revision_number' => '0');
	}
	
	public static function instance(){
		if(self::$peerInstance == null){
			$className = "DB_ForumPostPeer";
			self::$peerInstance = new $className();
		}
		return self::$peerInstance;
	}

}