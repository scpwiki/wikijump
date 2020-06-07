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
 * Base class mapped to the database table simpletodo_list.
 */
class SimpletodoListBase extends BaseDBObject {

	protected function internalInit(){
		$this->tableName='simpletodo_list';
		$this->peerName = 'DB\\SimpletodoListPeer';
		$this->primaryKeyName = 'list_id';
		$this->fieldNames = array( 'list_id' ,  'site_id' ,  'label' ,  'title' ,  'data' );

		//$this->fieldDefaultValues=
	}






	public function getListId() {
		return $this->getFieldValue('list_id');
	}

	public function setListId($v1, $raw=false) {
		$this->setFieldValue('list_id', $v1, $raw);
	}


	public function getSiteId() {
		return $this->getFieldValue('site_id');
	}

	public function setSiteId($v1, $raw=false) {
		$this->setFieldValue('site_id', $v1, $raw);
	}


	public function getLabel() {
		return $this->getFieldValue('label');
	}

	public function setLabel($v1, $raw=false) {
		$this->setFieldValue('label', $v1, $raw);
	}


	public function getTitle() {
		return $this->getFieldValue('title');
	}

	public function setTitle($v1, $raw=false) {
		$this->setFieldValue('title', $v1, $raw);
	}


	public function getData() {
		return $this->getFieldValue('data');
	}

	public function setData($v1, $raw=false) {
		$this->setFieldValue('data', $v1, $raw);
	}




}
