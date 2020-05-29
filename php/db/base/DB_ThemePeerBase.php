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
 * Base peer class mapped to the database table theme.
 */
class DB_ThemePeerBase extends BaseDBPeer {
	public static $peerInstance;
	
	protected function internalInit(){
		$this->tableName='theme';
		$this->objectName='DB_Theme';
		$this->primaryKeyName = 'theme_id';
		$this->fieldNames = array( 'theme_id' ,  'name' ,  'unix_name' ,  'abstract' ,  'extends_theme_id' ,  'variant_of_theme_id' ,  'custom' ,  'site_id' ,  'use_side_bar' ,  'use_top_bar' ,  'sort_index' ,  'sync_page_name' ,  'revision_number' );
		$this->fieldTypes = array( 'theme_id' => 'serial',  'name' => 'varchar(100)',  'unix_name' => 'varchar(100)',  'abstract' => 'boolean',  'extends_theme_id' => 'int',  'variant_of_theme_id' => 'int',  'custom' => 'boolean',  'site_id' => 'int',  'use_side_bar' => 'boolean',  'use_top_bar' => 'boolean',  'sort_index' => 'int',  'sync_page_name' => 'varchar(100)',  'revision_number' => 'int');
		$this->defaultValues = array( 'abstract' => 'false',  'custom' => 'false',  'use_side_bar' => 'true',  'use_top_bar' => 'true',  'sort_index' => '0',  'revision_number' => '0');
	}
	
	public static function instance(){
		if(self::$peerInstance == null){
			$className = "DB_ThemePeer";
			self::$peerInstance = new $className();
		}
		return self::$peerInstance;
	}

}