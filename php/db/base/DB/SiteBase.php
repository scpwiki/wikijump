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
 * Base class mapped to the database table site.
 */
class SiteBase extends BaseDBObject {

	protected function internalInit(){
		$this->tableName='site';
		$this->peerName = 'DB\\SitePeer';
		$this->primaryKeyName = 'site_id';
		$this->fieldNames = array( 'site_id' ,  'name' ,  'subtitle' ,  'unix_name' ,  'description' ,  'language' ,  'date_created' ,  'custom_domain' ,  'default_page' ,  'visible' ,  'private' ,  'deleted' );

		//$this->fieldDefaultValues=
	}






	public function getSiteId() {
		return $this->getFieldValue('site_id');
	}

	public function setSiteId($v1, $raw=false) {
		$this->setFieldValue('site_id', $v1, $raw);
	}


	public function getName() {
		return $this->getFieldValue('name');
	}

	public function setName($v1, $raw=false) {
		$this->setFieldValue('name', $v1, $raw);
	}


	public function getSubtitle() {
		return $this->getFieldValue('subtitle');
	}

	public function setSubtitle($v1, $raw=false) {
		$this->setFieldValue('subtitle', $v1, $raw);
	}


	public function getUnixName() {
		return $this->getFieldValue('unix_name');
	}

	public function setUnixName($v1, $raw=false) {
		$this->setFieldValue('unix_name', $v1, $raw);
	}


	public function getDescription() {
		return $this->getFieldValue('description');
	}

	public function setDescription($v1, $raw=false) {
		$this->setFieldValue('description', $v1, $raw);
	}


	public function getLanguage() {
		return $this->getFieldValue('language');
	}

	public function setLanguage($v1, $raw=false) {
		$this->setFieldValue('language', $v1, $raw);
	}


	public function getDateCreated() {
		return $this->getFieldValue('date_created');
	}

	public function setDateCreated($v1, $raw=false) {
		$this->setFieldValue('date_created', $v1, $raw);
	}


	public function getCustomDomain() {
		return $this->getFieldValue('custom_domain');
	}

	public function setCustomDomain($v1, $raw=false) {
		$this->setFieldValue('custom_domain', $v1, $raw);
	}


	public function getDefaultPage() {
		return $this->getFieldValue('default_page');
	}

	public function setDefaultPage($v1, $raw=false) {
		$this->setFieldValue('default_page', $v1, $raw);
	}


	public function getVisible() {
		return $this->getFieldValue('visible');
	}

	public function setVisible($v1, $raw=false) {
		$this->setFieldValue('visible', $v1, $raw);
	}


	public function getPrivate() {
		return $this->getFieldValue('private');
	}

	public function setPrivate($v1, $raw=false) {
		$this->setFieldValue('private', $v1, $raw);
	}


	public function getDeleted() {
		return $this->getFieldValue('deleted');
	}

	public function setDeleted($v1, $raw=false) {
		$this->setFieldValue('deleted', $v1, $raw);
	}




}
