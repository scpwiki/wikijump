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
 * Base class mapped to the database table comment_revision.
 */
class CommentRevisionBase extends BaseDBObject {

	protected function internalInit(){
		$this->tableName='comment_revision';
		$this->peerName = 'DB\\CommentRevisionPeer';
		$this->primaryKeyName = 'revision_id';
		$this->fieldNames = array( 'revision_id' ,  'comment_id' ,  'user_id' ,  'user_string' ,  'text' ,  'title' ,  'date' );

		//$this->fieldDefaultValues=
	}






	public function getRevisionId() {
		return $this->getFieldValue('revision_id');
	}

	public function setRevisionId($v1, $raw=false) {
		$this->setFieldValue('revision_id', $v1, $raw);
	}


	public function getCommentId() {
		return $this->getFieldValue('comment_id');
	}

	public function setCommentId($v1, $raw=false) {
		$this->setFieldValue('comment_id', $v1, $raw);
	}


	public function getUserId() {
		return $this->getFieldValue('user_id');
	}

	public function setUserId($v1, $raw=false) {
		$this->setFieldValue('user_id', $v1, $raw);
	}


	public function getUserString() {
		return $this->getFieldValue('user_string');
	}

	public function setUserString($v1, $raw=false) {
		$this->setFieldValue('user_string', $v1, $raw);
	}


	public function getText() {
		return $this->getFieldValue('text');
	}

	public function setText($v1, $raw=false) {
		$this->setFieldValue('text', $v1, $raw);
	}


	public function getTitle() {
		return $this->getFieldValue('title');
	}

	public function setTitle($v1, $raw=false) {
		$this->setFieldValue('title', $v1, $raw);
	}


	public function getDate() {
		return $this->getFieldValue('date');
	}

	public function setDate($v1, $raw=false) {
		$this->setFieldValue('date', $v1, $raw);
	}




}
