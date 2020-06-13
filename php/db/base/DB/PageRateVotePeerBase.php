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
 * Base peer class mapped to the database table page_rate_vote.
 */
class PageRateVotePeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='page_rate_vote';
        $this->objectName='DB\\PageRateVote';
        $this->primaryKeyName = 'rate_id';
        $this->fieldNames = array( 'rate_id' ,  'user_id' ,  'page_id' ,  'rate' ,  'date' );
        $this->fieldTypes = array( 'rate_id' => 'serial',  'user_id' => 'int',  'page_id' => 'int',  'rate' => 'int',  'date' => 'timestamp');
        $this->defaultValues = array( 'rate' => '1');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            $className = "DB\\PageRateVotePeer";
            self::$peerInstance = new $className();
        }
        return self::$peerInstance;
    }
}
