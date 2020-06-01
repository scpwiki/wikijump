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

use BaseDBObject;



 
/**
 * Base class mapped to the database table site_viewer.
 */
class SiteViewerBase extends BaseDBObject {

	protected function internalInit(){
		$this->tableName='site_viewer';
		$this->peerName = 'DB\\SiteViewerPeer';
		$this->primaryKeyName = 'viewer_id';
		$this->fieldNames = array( 'viewer_id' ,  'site_id' ,  'user_id' );
		
		//$this->fieldDefaultValues=
	}


	
		
	
		
	public function getViewerId() {
		return $this->getFieldValue('viewer_id');
	}
	
	public function setViewerId($v1, $raw=false) {
		$this->setFieldValue('viewer_id', $v1, $raw); 
	}
	
		
	public function getSiteId() {
		return $this->getFieldValue('site_id');
	}
	
	public function setSiteId($v1, $raw=false) {
		$this->setFieldValue('site_id', $v1, $raw); 
	}
	
		
	public function getUserId() {
		return $this->getFieldValue('user_id');
	}
	
	public function setUserId($v1, $raw=false) {
		$this->setFieldValue('user_id', $v1, $raw); 
	}
	
		
	

}
