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
 * Base class mapped to the database table log_event.
 */
class LogEventBase extends BaseDBObject {

	protected function internalInit(){
		$this->tableName='log_event';
		$this->peerName = 'DB\\LogEventPeer';
		$this->primaryKeyName = 'event_id';
		$this->fieldNames = array( 'event_id' ,  'date' ,  'user_id' ,  'ip' ,  'proxy' ,  'type' ,  'site_id' ,  'page_id' ,  'revision_id' ,  'thread_id' ,  'post_id' ,  'user_agent' ,  'text' );

		//$this->fieldDefaultValues=
	}






	public function getEventId() {
		return $this->getFieldValue('event_id');
	}

	public function setEventId($v1, $raw=false) {
		$this->setFieldValue('event_id', $v1, $raw);
	}


	public function getDate() {
		return $this->getFieldValue('date');
	}

	public function setDate($v1, $raw=false) {
		$this->setFieldValue('date', $v1, $raw);
	}


	public function getUserId() {
		return $this->getFieldValue('user_id');
	}

	public function setUserId($v1, $raw=false) {
		$this->setFieldValue('user_id', $v1, $raw);
	}


	public function getIp() {
		return $this->getFieldValue('ip');
	}

	public function setIp($v1, $raw=false) {
		$this->setFieldValue('ip', $v1, $raw);
	}


	public function getProxy() {
		return $this->getFieldValue('proxy');
	}

	public function setProxy($v1, $raw=false) {
		$this->setFieldValue('proxy', $v1, $raw);
	}


	public function getType() {
		return $this->getFieldValue('type');
	}

	public function setType($v1, $raw=false) {
		$this->setFieldValue('type', $v1, $raw);
	}


	public function getSiteId() {
		return $this->getFieldValue('site_id');
	}

	public function setSiteId($v1, $raw=false) {
		$this->setFieldValue('site_id', $v1, $raw);
	}


	public function getPageId() {
		return $this->getFieldValue('page_id');
	}

	public function setPageId($v1, $raw=false) {
		$this->setFieldValue('page_id', $v1, $raw);
	}


	public function getRevisionId() {
		return $this->getFieldValue('revision_id');
	}

	public function setRevisionId($v1, $raw=false) {
		$this->setFieldValue('revision_id', $v1, $raw);
	}


	public function getThreadId() {
		return $this->getFieldValue('thread_id');
	}

	public function setThreadId($v1, $raw=false) {
		$this->setFieldValue('thread_id', $v1, $raw);
	}


	public function getPostId() {
		return $this->getFieldValue('post_id');
	}

	public function setPostId($v1, $raw=false) {
		$this->setFieldValue('post_id', $v1, $raw);
	}


	public function getUserAgent() {
		return $this->getFieldValue('user_agent');
	}

	public function setUserAgent($v1, $raw=false) {
		$this->setFieldValue('user_agent', $v1, $raw);
	}


	public function getText() {
		return $this->getFieldValue('text');
	}

	public function setText($v1, $raw=false) {
		$this->setFieldValue('text', $v1, $raw);
	}




}
