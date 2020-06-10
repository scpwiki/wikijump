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
 * Base class mapped to the database table ozone_permission.
 */
class OzonePermissionBase extends BaseDBObject {

    protected function internalInit(){
        $this->tableName='ozone_permission';
        $this->peerName = 'DB\\OzonePermissionPeer';
        $this->primaryKeyName = 'permission_id';
        $this->fieldNames = array( 'permission_id' ,  'name' ,  'description' );

        //$this->fieldDefaultValues=
    }






    public function getPermissionId() {
        return $this->getFieldValue('permission_id');
    }

    public function setPermissionId($v1, $raw=false) {
        $this->setFieldValue('permission_id', $v1, $raw);
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




}
