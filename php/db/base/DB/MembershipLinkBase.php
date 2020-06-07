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
 * Base class mapped to the database table membership_link.
 */
class MembershipLinkBase extends BaseDBObject {

	protected function internalInit(){
		$this->tableName='membership_link';
		$this->peerName = 'DB\\MembershipLinkPeer';
		$this->primaryKeyName = 'link_id';
		$this->fieldNames = array( 'link_id' ,  'site_id' ,  'by_user_id' ,  'user_id' ,  'date' ,  'type' );

		//$this->fieldDefaultValues=
	}






	public function getLinkId() {
		return $this->getFieldValue('link_id');
	}

	public function setLinkId($v1, $raw=false) {
		$this->setFieldValue('link_id', $v1, $raw);
	}


	public function getSiteId() {
		return $this->getFieldValue('site_id');
	}

	public function setSiteId($v1, $raw=false) {
		$this->setFieldValue('site_id', $v1, $raw);
	}


	public function getByUserId() {
		return $this->getFieldValue('by_user_id');
	}

	public function setByUserId($v1, $raw=false) {
		$this->setFieldValue('by_user_id', $v1, $raw);
	}


	public function getUserId() {
		return $this->getFieldValue('user_id');
	}

	public function setUserId($v1, $raw=false) {
		$this->setFieldValue('user_id', $v1, $raw);
	}


	public function getDate() {
		return $this->getFieldValue('date');
	}

	public function setDate($v1, $raw=false) {
		$this->setFieldValue('date', $v1, $raw);
	}


	public function getType() {
		return $this->getFieldValue('type');
	}

	public function setType($v1, $raw=false) {
		$this->setFieldValue('type', $v1, $raw);
	}




}
