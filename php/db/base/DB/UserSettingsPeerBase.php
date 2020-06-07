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
 * Base peer class mapped to the database table user_settings.
 */
class UserSettingsPeerBase extends BaseDBPeer {
	public static $peerInstance;

	protected function internalInit(){
		$this->tableName='user_settings';
		$this->objectName='DB\\UserSettings';
		$this->primaryKeyName = 'user_id';
		$this->fieldNames = array( 'user_id' ,  'receive_invitations' ,  'receive_pm' ,  'receive_newsletter' ,  'receive_digest' ,  'notify_online' ,  'notify_feed' ,  'notify_email' ,  'allow_site_newsletters_default' ,  'max_sites_admin' );
		$this->fieldTypes = array( 'user_id' => 'int',  'receive_invitations' => 'boolean',  'receive_pm' => 'char(5)',  'receive_newsletter' => 'boolean',  'receive_digest' => 'boolean',  'notify_online' => 'varchar(512)',  'notify_feed' => 'varchar(512)',  'notify_email' => 'varchar(512)',  'allow_site_newsletters_default' => 'boolean',  'max_sites_admin' => 'int');
		$this->defaultValues = array( 'receive_invitations' => 'true',  'receive_pm' => 'a',  'receive_newsletter' => 'true',  'receive_digest' => 'true',  'notify_online' => '*',  'notify_feed' => '*',  'allow_site_newsletters_default' => 'true',  'max_sites_admin' => '3');
	}

	public static function instance(){
		if(self::$peerInstance == null){
			$className = "DB\\UserSettingsPeer";
			self::$peerInstance = new $className();
		}
		return self::$peerInstance;
	}

}