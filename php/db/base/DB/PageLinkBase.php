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
 * Base class mapped to the database table page_link.
 */
class PageLinkBase extends BaseDBObject {

	protected function internalInit(){
		$this->tableName='page_link';
		$this->peerName = 'DB\\PageLinkPeer';
		$this->primaryKeyName = 'link_id';
		$this->fieldNames = array( 'link_id' ,  'site_id' ,  'from_page_id' ,  'to_page_id' ,  'to_page_name' );

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


	public function getFromPageId() {
		return $this->getFieldValue('from_page_id');
	}

	public function setFromPageId($v1, $raw=false) {
		$this->setFieldValue('from_page_id', $v1, $raw);
	}


	public function getToPageId() {
		return $this->getFieldValue('to_page_id');
	}

	public function setToPageId($v1, $raw=false) {
		$this->setFieldValue('to_page_id', $v1, $raw);
	}


	public function getToPageName() {
		return $this->getFieldValue('to_page_name');
	}

	public function setToPageName($v1, $raw=false) {
		$this->setFieldValue('to_page_name', $v1, $raw);
	}




}
