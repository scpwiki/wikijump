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

use BaseDBPeer;




/**
 * Base peer class mapped to the database table petition_signature.
 */
class PetitionSignaturePeerBase extends BaseDBPeer {
    public static $peerInstance;

    protected function internalInit(){
        $this->tableName='petition_signature';
        $this->objectName='DB\\PetitionSignature';
        $this->primaryKeyName = 'signature_id';
        $this->fieldNames = array( 'signature_id' ,  'campaign_id' ,  'first_name' ,  'last_name' ,  'address1' ,  'address2' ,  'zip' ,  'city' ,  'state' ,  'country' ,  'country_code' ,  'comments' ,  'email' ,  'confirmed' ,  'confirmation_hash' ,  'confirmation_url' ,  'date' );
        $this->fieldTypes = array( 'signature_id' => 'serial',  'campaign_id' => 'int',  'first_name' => 'varchar(256)',  'last_name' => 'varchar(256)',  'address1' => 'varchar(256)',  'address2' => 'varchar(256)',  'zip' => 'varchar(256)',  'city' => 'varchar(256)',  'state' => 'varchar(256)',  'country' => 'varchar(256)',  'country_code' => 'varchar(8)',  'comments' => 'text',  'email' => 'varchar(256)',  'confirmed' => 'boolean',  'confirmation_hash' => 'varchar(256)',  'confirmation_url' => 'varchar(256)',  'date' => 'timestamp');
        $this->defaultValues = array( 'confirmed' => 'false');
    }

    public static function instance(){
        if(self::$peerInstance == null){
            $className = "DB\\PetitionSignaturePeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }

}