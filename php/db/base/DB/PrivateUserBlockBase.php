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
 * Base class mapped to the database table private_user_block.
 */
class PrivateUserBlockBase extends BaseDBObject {

	protected function internalInit(){
		$this->tableName='private_user_block';
		$this->peerName = 'DB\\PrivateUserBlockPeer';
		$this->primaryKeyName = 'block_id';
		$this->fieldNames = array( 'block_id' ,  'user_id' ,  'blocked_user_id' );

		//$this->fieldDefaultValues=
	}






	public function getBlockId() {
		return $this->getFieldValue('block_id');
	}

	public function setBlockId($v1, $raw=false) {
		$this->setFieldValue('block_id', $v1, $raw);
	}


	public function getUserId() {
		return $this->getFieldValue('user_id');
	}

	public function setUserId($v1, $raw=false) {
		$this->setFieldValue('user_id', $v1, $raw);
	}


	public function getBlockedUserId() {
		return $this->getFieldValue('blocked_user_id');
	}

	public function setBlockedUserId($v1, $raw=false) {
		$this->setFieldValue('blocked_user_id', $v1, $raw);
	}




}
