<?php
/**
 * Wikidot - free wiki collaboration software
 * Copyright (c) 2008-2020, Wikidot Inc., SCP Wiki Technical Team
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
 * @copyright Copyright (c) 2008-2020, Wikidot Inc., SCP Wiki Technical Team
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

namespace DB;

use BaseDBPeer;

/**
 * Base peer class mapped to the database table global_ip_block.
 */
class GlobalIpBlockPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='global_ip_block';
        $this->objectName='DB\\GlobalIpBlock';
        $this->primaryKeyName = 'block_id';
        $this->fieldNames = array( 'block_id' ,  'address' ,  'flag_proxy' ,  'reason' ,  'flag_total' ,  'date_blocked' );
        $this->fieldTypes = array( 'block_id' => 'serial',  'address' => 'inet',  'flag_proxy' => 'boolean',  'reason' => 'text',  'flag_total' => 'boolean',  'date_blocked' => 'timestamp');
        $this->defaultValues = array( 'flag_proxy' => 'false',  'flag_total' => 'false');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\GlobalIpBlockPeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
