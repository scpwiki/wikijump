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
 
/**
 * Base peer class mapped to the database table page_revision.
 */
class DB_PageRevisionPeerBase extends BaseDBPeer {
	public static $peerInstance;
	
	protected function internalInit(){
		$this->tableName='page_revision';
		$this->objectName='DB_PageRevision';
		$this->primaryKeyName = 'revision_id';
		$this->fieldNames = array( 'revision_id' ,  'page_id' ,  'site_id' ,  'source_id' ,  'metadata_id' ,  'flags' ,  'flag_text' ,  'flag_title' ,  'flag_file' ,  'flag_rename' ,  'flag_meta' ,  'flag_new' ,  'flag_new_site' ,  'since_full_source' ,  'diff_source' ,  'revision_number' ,  'date_last_edited' ,  'user_id' ,  'user_string' ,  'comments' );
		$this->fieldTypes = array( 'revision_id' => 'serial',  'page_id' => 'int',  'site_id' => 'int',  'source_id' => 'int',  'metadata_id' => 'int',  'flags' => 'varchar(100)',  'flag_text' => 'boolean',  'flag_title' => 'boolean',  'flag_file' => 'boolean',  'flag_rename' => 'boolean',  'flag_meta' => 'boolean',  'flag_new' => 'boolean',  'flag_new_site' => 'boolean',  'since_full_source' => 'int',  'diff_source' => 'boolean',  'revision_number' => 'int',  'date_last_edited' => 'timestamp',  'user_id' => 'int',  'user_string' => 'varchar(80)',  'comments' => 'text');
		$this->defaultValues = array( 'flag_text' => 'false',  'flag_title' => 'false',  'flag_file' => 'false',  'flag_rename' => 'false',  'flag_meta' => 'false',  'flag_new' => 'false',  'flag_new_site' => 'false',  'since_full_source' => '0',  'diff_source' => 'false',  'revision_number' => '0');
	}
	
	public static function instance(){
		if(self::$peerInstance == null){
			$className = "DB_PageRevisionPeer";
			self::$peerInstance = new $className();
		}
		return self::$peerInstance;
	}

}