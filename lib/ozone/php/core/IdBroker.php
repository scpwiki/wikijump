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
 * @category Ozone
 * @package Ozone_Db
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */


use DB\IdBrokerPeer;

/**
 * Id broker utility.
 *
 */
class IdBroker{

    public function nextId($tableName){
        $db = Database::connection();
        $idbpeer = IdBrokerPeer::instance();
        $db->begin();
        $t = $idbpeer->selectOneByExplicitQuery("WHERE table_name = '$tableName' FOR UPDATE");
        $index = $t->getNextFreeIndex();
        $t->setNextFreeIndex($index+1);
        $t->save();
        $db->commit();
        return $index;
    }

}
