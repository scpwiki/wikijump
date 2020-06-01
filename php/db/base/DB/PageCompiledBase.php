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
 * Base class mapped to the database table page_compiled.
 */
class PageCompiledBase extends BaseDBObject {

	protected function internalInit(){
		$this->tableName='page_compiled';
		$this->peerName = 'DB\\PageCompiledPeer';
		$this->primaryKeyName = 'page_id';
		$this->fieldNames = array( 'page_id' ,  'text' ,  'date_compiled' );
		
		//$this->fieldDefaultValues=
	}


	
		
	
		
	public function getPageId() {
		return $this->getFieldValue('page_id');
	}
	
	public function setPageId($v1, $raw=false) {
		$this->setFieldValue('page_id', $v1, $raw); 
	}
	
		
	public function getText() {
		return $this->getFieldValue('text');
	}
	
	public function setText($v1, $raw=false) {
		$this->setFieldValue('text', $v1, $raw); 
	}
	
		
	public function getDateCompiled() {
		return $this->getFieldValue('date_compiled');
	}
	
	public function setDateCompiled($v1, $raw=false) {
		$this->setFieldValue('date_compiled', $v1, $raw); 
	}
	
		
	

}
