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
 * Base class mapped to the database table theme.
 */
class ThemeBase extends BaseDBObject {

	protected function internalInit(){
		$this->tableName='theme';
		$this->peerName = 'DB\\ThemePeer';
		$this->primaryKeyName = 'theme_id';
		$this->fieldNames = array( 'theme_id' ,  'name' ,  'unix_name' ,  'abstract' ,  'extends_theme_id' ,  'variant_of_theme_id' ,  'custom' ,  'site_id' ,  'use_side_bar' ,  'use_top_bar' ,  'sort_index' ,  'sync_page_name' ,  'revision_number' );

		//$this->fieldDefaultValues=
	}






	public function getThemeId() {
		return $this->getFieldValue('theme_id');
	}

	public function setThemeId($v1, $raw=false) {
		$this->setFieldValue('theme_id', $v1, $raw);
	}


	public function getName() {
		return $this->getFieldValue('name');
	}

	public function setName($v1, $raw=false) {
		$this->setFieldValue('name', $v1, $raw);
	}


	public function getUnixName() {
		return $this->getFieldValue('unix_name');
	}

	public function setUnixName($v1, $raw=false) {
		$this->setFieldValue('unix_name', $v1, $raw);
	}


	public function getAbstract() {
		return $this->getFieldValue('abstract');
	}

	public function setAbstract($v1, $raw=false) {
		$this->setFieldValue('abstract', $v1, $raw);
	}


	public function getExtendsThemeId() {
		return $this->getFieldValue('extends_theme_id');
	}

	public function setExtendsThemeId($v1, $raw=false) {
		$this->setFieldValue('extends_theme_id', $v1, $raw);
	}


	public function getVariantOfThemeId() {
		return $this->getFieldValue('variant_of_theme_id');
	}

	public function setVariantOfThemeId($v1, $raw=false) {
		$this->setFieldValue('variant_of_theme_id', $v1, $raw);
	}


	public function getCustom() {
		return $this->getFieldValue('custom');
	}

	public function setCustom($v1, $raw=false) {
		$this->setFieldValue('custom', $v1, $raw);
	}


	public function getSiteId() {
		return $this->getFieldValue('site_id');
	}

	public function setSiteId($v1, $raw=false) {
		$this->setFieldValue('site_id', $v1, $raw);
	}


	public function getUseSideBar() {
		return $this->getFieldValue('use_side_bar');
	}

	public function setUseSideBar($v1, $raw=false) {
		$this->setFieldValue('use_side_bar', $v1, $raw);
	}


	public function getUseTopBar() {
		return $this->getFieldValue('use_top_bar');
	}

	public function setUseTopBar($v1, $raw=false) {
		$this->setFieldValue('use_top_bar', $v1, $raw);
	}


	public function getSortIndex() {
		return $this->getFieldValue('sort_index');
	}

	public function setSortIndex($v1, $raw=false) {
		$this->setFieldValue('sort_index', $v1, $raw);
	}


	public function getSyncPageName() {
		return $this->getFieldValue('sync_page_name');
	}

	public function setSyncPageName($v1, $raw=false) {
		$this->setFieldValue('sync_page_name', $v1, $raw);
	}


	public function getRevisionNumber() {
		return $this->getFieldValue('revision_number');
	}

	public function setRevisionNumber($v1, $raw=false) {
		$this->setFieldValue('revision_number', $v1, $raw);
	}




}
