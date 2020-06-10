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
 * Base class mapped to the database table openid_entry.
 */
class OpenidEntryBase extends BaseDBObject {

    protected function internalInit(){
        $this->tableName='openid_entry';
        $this->peerName = 'DB\\OpenidEntryPeer';
        $this->primaryKeyName = 'openid_id';
        $this->fieldNames = array( 'openid_id' ,  'site_id' ,  'page_id' ,  'type' ,  'user_id' ,  'url' ,  'server_url' );

        //$this->fieldDefaultValues=
    }






    public function getOpenidId() {
        return $this->getFieldValue('openid_id');
    }

    public function setOpenidId($v1, $raw=false) {
        $this->setFieldValue('openid_id', $v1, $raw);
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


    public function getType() {
        return $this->getFieldValue('type');
    }

    public function setType($v1, $raw=false) {
        $this->setFieldValue('type', $v1, $raw);
    }


    public function getUserId() {
        return $this->getFieldValue('user_id');
    }

    public function setUserId($v1, $raw=false) {
        $this->setFieldValue('user_id', $v1, $raw);
    }


    public function getUrl() {
        return $this->getFieldValue('url');
    }

    public function setUrl($v1, $raw=false) {
        $this->setFieldValue('url', $v1, $raw);
    }


    public function getServerUrl() {
        return $this->getFieldValue('server_url');
    }

    public function setServerUrl($v1, $raw=false) {
        $this->setFieldValue('server_url', $v1, $raw);
    }




}
