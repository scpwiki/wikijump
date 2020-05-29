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
 * Base peer class mapped to the database table log_event.
 */
class DB_LogEventPeerBase extends BaseDBPeer {
	public static $peerInstance;
	
	protected function internalInit(){
		$this->tableName='log_event';
		$this->objectName='DB_LogEvent';
		$this->primaryKeyName = 'event_id';
		$this->fieldNames = array( 'event_id' ,  'date' ,  'user_id' ,  'ip' ,  'proxy' ,  'type' ,  'site_id' ,  'page_id' ,  'revision_id' ,  'thread_id' ,  'post_id' ,  'user_agent' ,  'text' );
		$this->fieldTypes = array( 'event_id' => 'bigserial',  'date' => 'timestamp',  'user_id' => 'int',  'ip' => 'inet',  'proxy' => 'inet',  'type' => 'varchar(256)',  'site_id' => 'int',  'page_id' => 'int',  'revision_id' => 'int',  'thread_id' => 'int',  'post_id' => 'int',  'user_agent' => 'varchar(512)',  'text' => 'text');
		$this->defaultValues = array();
	}
	
	public static function instance(){
		if(self::$peerInstance == null){
			$className = "DB_LogEventPeer";
			self::$peerInstance = new $className();
		}
		return self::$peerInstance;
	}

}