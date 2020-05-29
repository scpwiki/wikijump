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
 * Base peer class mapped to the database table private_message.
 */
class DB_PrivateMessagePeerBase extends BaseDBPeer {
	public static $peerInstance;
	
	protected function internalInit(){
		$this->tableName='private_message';
		$this->objectName='DB_PrivateMessage';
		$this->primaryKeyName = 'message_id';
		$this->fieldNames = array( 'message_id' ,  'from_user_id' ,  'to_user_id' ,  'subject' ,  'body' ,  'date' ,  'flag' ,  'flag_new' );
		$this->fieldTypes = array( 'message_id' => 'serial',  'from_user_id' => 'int',  'to_user_id' => 'int',  'subject' => 'varchar(256)',  'body' => 'text',  'date' => 'timestamp',  'flag' => 'int',  'flag_new' => 'boolean');
		$this->defaultValues = array( 'flag' => '0',  'flag_new' => 'true');
	}
	
	public static function instance(){
		if(self::$peerInstance == null){
			$className = "DB_PrivateMessagePeer";
			self::$peerInstance = new $className();
		}
		return self::$peerInstance;
	}

}