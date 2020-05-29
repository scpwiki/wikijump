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
 * Base class mapped to the database table theme_preview.
 */
class DB_ThemePreviewBase extends BaseDBObject {

	protected function internalInit(){
		$this->tableName='theme_preview';
		$this->peerName = 'DB_ThemePreviewPeer';
		$this->primaryKeyName = 'theme_id';
		$this->fieldNames = array( 'theme_id' ,  'body' );
		
		//$this->fieldDefaultValues=
	}


	
		
	
		
	public function getThemeId() {
		return $this->getFieldValue('theme_id');
	}
	
	public function setThemeId($v1, $raw=false) {
		$this->setFieldValue('theme_id', $v1, $raw); 
	}
	
		
	public function getBody() {
		return $this->getFieldValue('body');
	}
	
	public function setBody($v1, $raw=false) {
		$this->setFieldValue('body', $v1, $raw); 
	}
	
		
	

}
