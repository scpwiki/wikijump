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
 * Base class mapped to the database table form_submission_key.
 */
class FormSubmissionKeyBase extends BaseDBObject {

	protected function internalInit(){
		$this->tableName='form_submission_key';
		$this->peerName = 'DB_FormSubmissionKeyPeer';
		$this->primaryKeyName = 'key_id';
		$this->fieldNames = array( 'key_id' ,  'date_submitted' );
		
		//$this->fieldDefaultValues=
	}


	
		
	
		
	public function getKeyId() {
		return $this->getFieldValue('key_id');
	}
	
	public function setKeyId($v1, $raw=false) {
		$this->setFieldValue('key_id', $v1, $raw); 
	}
	
		
	public function getDateSubmitted() {
		return $this->getFieldValue('date_submitted');
	}
	
	public function setDateSubmitted($v1, $raw=false) {
		$this->setFieldValue('date_submitted', $v1, $raw); 
	}
	
		
	

}
