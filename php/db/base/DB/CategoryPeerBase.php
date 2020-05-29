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
 * Base peer class mapped to the database table category.
 */
class CategoryPeerBase extends BaseDBPeer {
	public static $peerInstance;
	
	protected function internalInit(){
		$this->tableName='category';
		$this->objectName='DB_Category';
		$this->primaryKeyName = 'category_id';
		$this->fieldNames = array( 'category_id' ,  'site_id' ,  'name' ,  'theme_default' ,  'theme_id' ,  'theme_external_url' ,  'permissions_default' ,  'permissions' ,  'license_default' ,  'license_id' ,  'license_other' ,  'nav_default' ,  'top_bar_page_name' ,  'side_bar_page_name' ,  'template_id' ,  'per_page_discussion' ,  'per_page_discussion_default' ,  'rating' ,  'category_template_id' ,  'autonumerate' ,  'page_title_template' ,  'enable_pingback_out' ,  'enable_pingback_in' );
		$this->fieldTypes = array( 'category_id' => 'serial',  'site_id' => 'int',  'name' => 'varchar(80)',  'theme_default' => 'boolean',  'theme_id' => 'int',  'theme_external_url' => 'varchar(512)',  'permissions_default' => 'boolean',  'permissions' => 'varchar(200)',  'license_default' => 'boolean',  'license_id' => 'int',  'license_other' => 'varchar(300)',  'nav_default' => 'boolean',  'top_bar_page_name' => 'varchar(128)',  'side_bar_page_name' => 'varchar(128)',  'template_id' => 'int',  'per_page_discussion' => 'boolean',  'per_page_discussion_default' => 'boolean',  'rating' => 'varchar(10)',  'category_template_id' => 'int',  'autonumerate' => 'boolean',  'page_title_template' => 'varchar(256)',  'enable_pingback_out' => 'boolean',  'enable_pingback_in' => 'boolean');
		$this->defaultValues = array( 'theme_default' => 'true',  'permissions_default' => 'true',  'license_default' => 'true',  'nav_default' => 'true',  'per_page_discussion_default' => 'true',  'autonumerate' => 'false',  'enable_pingback_out' => 'false',  'enable_pingback_in' => 'false');
	}
	
	public static function instance(){
		if(self::$peerInstance == null){
			$className = "DB_CategoryPeer";
			self::$peerInstance = new $className();
		}
		return self::$peerInstance;
	}

}