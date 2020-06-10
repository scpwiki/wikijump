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
 * Base class mapped to the database table user_settings.
 */
class UserSettingsBase extends BaseDBObject {

    protected function internalInit(){
        $this->tableName='user_settings';
        $this->peerName = 'DB\\UserSettingsPeer';
        $this->primaryKeyName = 'user_id';
        $this->fieldNames = array( 'user_id' ,  'receive_invitations' ,  'receive_pm' ,  'receive_newsletter' ,  'receive_digest' ,  'notify_online' ,  'notify_feed' ,  'notify_email' ,  'allow_site_newsletters_default' ,  'max_sites_admin' );

        //$this->fieldDefaultValues=
    }






    public function getUserId() {
        return $this->getFieldValue('user_id');
    }

    public function setUserId($v1, $raw=false) {
        $this->setFieldValue('user_id', $v1, $raw);
    }


    public function getReceiveInvitations() {
        return $this->getFieldValue('receive_invitations');
    }

    public function setReceiveInvitations($v1, $raw=false) {
        $this->setFieldValue('receive_invitations', $v1, $raw);
    }


    public function getReceivePm() {
        return $this->getFieldValue('receive_pm');
    }

    public function setReceivePm($v1, $raw=false) {
        $this->setFieldValue('receive_pm', $v1, $raw);
    }


    public function getReceiveNewsletter() {
        return $this->getFieldValue('receive_newsletter');
    }

    public function setReceiveNewsletter($v1, $raw=false) {
        $this->setFieldValue('receive_newsletter', $v1, $raw);
    }


    public function getReceiveDigest() {
        return $this->getFieldValue('receive_digest');
    }

    public function setReceiveDigest($v1, $raw=false) {
        $this->setFieldValue('receive_digest', $v1, $raw);
    }


    public function getNotifyOnline() {
        return $this->getFieldValue('notify_online');
    }

    public function setNotifyOnline($v1, $raw=false) {
        $this->setFieldValue('notify_online', $v1, $raw);
    }


    public function getNotifyFeed() {
        return $this->getFieldValue('notify_feed');
    }

    public function setNotifyFeed($v1, $raw=false) {
        $this->setFieldValue('notify_feed', $v1, $raw);
    }


    public function getNotifyEmail() {
        return $this->getFieldValue('notify_email');
    }

    public function setNotifyEmail($v1, $raw=false) {
        $this->setFieldValue('notify_email', $v1, $raw);
    }


    public function getAllowSiteNewslettersDefault() {
        return $this->getFieldValue('allow_site_newsletters_default');
    }

    public function setAllowSiteNewslettersDefault($v1, $raw=false) {
        $this->setFieldValue('allow_site_newsletters_default', $v1, $raw);
    }


    public function getMaxSitesAdmin() {
        return $this->getFieldValue('max_sites_admin');
    }

    public function setMaxSitesAdmin($v1, $raw=false) {
        $this->setFieldValue('max_sites_admin', $v1, $raw);
    }




}
