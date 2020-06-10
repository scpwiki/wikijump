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
 * Base class mapped to the database table license.
 */
class LicenseBase extends BaseDBObject {

    protected function internalInit(){
        $this->tableName='license';
        $this->peerName = 'DB\\LicensePeer';
        $this->primaryKeyName = 'license_id';
        $this->fieldNames = array( 'license_id' ,  'name' ,  'description' ,  'sort' );

        //$this->fieldDefaultValues=
    }






    public function getLicenseId() {
        return $this->getFieldValue('license_id');
    }

    public function setLicenseId($v1, $raw=false) {
        $this->setFieldValue('license_id', $v1, $raw);
    }


    public function getName() {
        return $this->getFieldValue('name');
    }

    public function setName($v1, $raw=false) {
        $this->setFieldValue('name', $v1, $raw);
    }


    public function getDescription() {
        return $this->getFieldValue('description');
    }

    public function setDescription($v1, $raw=false) {
        $this->setFieldValue('description', $v1, $raw);
    }


    public function getSort() {
        return $this->getFieldValue('sort');
    }

    public function setSort($v1, $raw=false) {
        $this->setFieldValue('sort', $v1, $raw);
    }




}
