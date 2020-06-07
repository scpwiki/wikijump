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
 * Base peer class mapped to the database table page_tag.
 */
class PageTagPeerBase extends BaseDBPeer {
	public static $peerInstance;

	protected function internalInit(){
		$this->tableName='page_tag';
		$this->objectName='DB\\PageTag';
		$this->primaryKeyName = 'tag_id';
		$this->fieldNames = array( 'tag_id' ,  'site_id' ,  'page_id' ,  'tag' );
		$this->fieldTypes = array( 'tag_id' => 'bigserial',  'site_id' => 'int',  'page_id' => 'int',  'tag' => 'varchar(64)');
		$this->defaultValues = array();
	}

	public static function instance(){
		if(self::$peerInstance == null){
			self::$peerInstance = new PageTagPeer();
		}
		return self::$peerInstance;
	}

}
