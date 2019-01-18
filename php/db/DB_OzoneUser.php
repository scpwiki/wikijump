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
 * @package Wikidot_Db
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

/**
 * Object Model class.
 *
 */
class DB_OzoneUser extends DB_OzoneUserBase {

    public function getProfile() {
        if (is_array($this->prefetched)) {
            if (in_array('profile', $this->prefetched)) {
                if (in_array('profile', $this->prefetchedObjects)) {
                    return $this->prefetchedObjects['profile'];
                } else {
                    $obj = new DB_Profile($this->sourceRow);
                    $obj->setNew(false);
                    $this->prefetchedObjects['profile'] = $obj;
                    return $obj;
                }
            }
        }
        return DB_ProfilePeer::instance()->selectByPrimaryKey($this->getUserId());
    }

    public function getSettings() {
        return DB_UserSettingsPeer::instance()->selectByPrimaryKey($this->getUserId());
    }

    public function getKarmaLevel() {
        $c = new Criteria();
        $c->add('user_id', $this->getUserId());
        $karma = DB_UserKarmaPeer::instance()->selectOne($c);
        if($karma){
            return $karma->getLevel();
        } else {
            return 0;
        }
    }
    
    public function save() {
        $memcache = Ozone::$memcache;
        $key = 'user..' . $this->getUserId();
        $memcache->delete($key);
        parent::save();
    }

}
