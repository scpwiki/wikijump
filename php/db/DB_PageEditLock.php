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
class DB_PageEditLock extends DB_PageEditLockBase {

    public function getConflicts() {
        $c = $this->getConflictsCriteria();
        $conflicts = DB_PageEditLockPeer::instance()->select($c);
        if (count($conflicts) == 0) {
            return null;
        } else {
            return $conflicts;
        }

    }

    public function deleteConflicts() {
        $c = $this->getConflictsCriteria();
        DB_PageEditLockPeer::instance()->delete($c);
    }

    public function getConflictsCriteria() {
        // get conflicting page locks

        // if lock for a new page...
        // anyway one should also check if a page does not exist
        // (done somewhere else)

        if ($this->getPageId() == null) {
            $c = new Criteria();
            $c->add("page_unix_name", $this->getPageUnixName());
            $c->add("site_id", $this->getSiteId());
            if ($this->getLockId() != null) {
                $c->add("lock_id", $this->getLockId(), "!=");
            }
        } else {
            
            // now if the page exists.

            if ($this->getMode() == "page") {
                // conflicts with any other type of lock for this page...
                $c = new Criteria();
                $c->add("page_id", $this->getPageId());
                if ($this->getLockId() != null) {
                    $c->add("lock_id", $this->getLockId(), "!=");
                }
            }
            
            if ($this->getMode() == "append") {
                // conflicts only with "page" mode
                $c = new Criteria();
                $c->add("page_id", $this->getPageId());
                if ($this->getLockId() != null) {
                    $c->add("lock_id", $this->getLockId(), "!=");
                }
                $c->add("mode", "page");
            }
            
            if ($this->getMode() == "section") {
                // conflicts with "page" mode and "section" mode when regions overlap
                $c = new Criteria();
                
                // create query by hand...
                $q = "page_id = " . $this->getPageId() . " ";
                if ($this->getLockId() != null) {
                    $q .= "AND lock_id != " . $this->getLockId() . " ";
                }
                $q .= "AND (mode = 'page' " . "OR (" . "mode = 'section' AND (" . "(range_start >= '" . db_escape_string($this->getRangeStart()) . "' AND range_start <= '" . db_escape_string($this->getRangeEnd()) . "') " . "OR (range_end >= '" . db_escape_string($this->getRangeStart()) . "' AND range_end <= '" . db_escape_string($this->getRangeEnd()) . "') " . "OR (range_start <= '" . db_escape_string($this->getRangeStart()) . "' AND range_end >= '" . db_escape_string($this->getRangeEnd()) . "')" . ")))";
                $c->setExplicitWhere($q);
            }
        }
        return $c;
    }

    public function getUser() {
        if ($this->getUserId() == 0) {
            return null;
        }
        if (is_array($this->prefetched)) {
            if (in_array('ozone_user', $this->prefetched)) {
                if (in_array('ozone_user', $this->prefetchedObjects)) {
                    return $this->prefetchedObjects['ozone_user'];
                } else {
                    $obj = new DB_OzoneUser($this->sourceRow);
                    $obj->setNew(false);
                    $this->prefetchedObjects['ozone_user'] = $obj;
                    return $obj;
                }
            }
        }
        return DB_OzoneUserPeer::instance()->selectByPrimaryKey($this->getUserId());
    
    }

    public function getUserOrString() {
        $user = $this->getUser();
        if ($user == null) {
            return $this->getUserString();
        } else {
            return $user;
        }
    
    }

    /**
     * Calculate number of seconds this lock will expire.
     */
    public function getExpireIn() {
        return $this->getDateLastAccessed()->getTimestamp() + 15 * 60 - time();
    }

    public function getStartedAgo() {
        return time() - $this->getDateStarted()->getTimestamp();
    }

    public function getOzoneUser() {
        return DB_OzoneUserPeer::instance()->selectByPrimaryKey($this->getUserId());
    }

}
