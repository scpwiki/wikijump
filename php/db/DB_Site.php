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
class DB_Site extends DB_SiteBase {

    public function getDomain() {
        if ($this->getCustomDomain() == null || $this->getCustomDomain() == "") {
            return $this->getUnixName() . "." . GlobalProperties::$URL_DOMAIN;
        } else {
            return $this->getCustomDomain();
        }
    }

    public function getSettings() {
        $key = "sitesettings.." . $this->getSiteId();
        $mc = OZONE::$memcache;
        $s = $mc->get($key);
        if (!$s) {
            $c = new Criteria();
            $c->add("site_id", $this->getSiteId());
            $s = DB_SiteSettingsPeer::instance()->selectOne($c);
            $mc->set($key, $s, 0, 864000);
        }
        return $s;
    }

    public function getSuperSettings() {

        $s = DB_SiteSuperSettingsPeer::instance()->selectByPrimaryKey($this->getSiteId());
        
        return $s;
    
    }

    public function getForumSettings() {
        $c = new Criteria();
        $c->add("site_id", $this->getSiteId());
        return DB_ForumSettingsPeer::instance()->selectOne($c);
    }

    public function save() {
        $memcache = Ozone::$memcache;
        $key = 'site..' . $this->getUnixName();
        $memcache->delete($key);
        $key = 'site_cd..' . $this->getCustomDomain();
        $memcache->delete($key);
        parent::save();
    }

    public function getLocalFilesPath(){
    	return WIKIDOT_ROOT . '/web/files--sites/'.$this->getUnixName();
    	
    	/* optional hashing */
    	$un = $this->getUnixName();
    	$p = substr($un,0,1) . '/' . substr($un,0,2) . '/' . $un;

    	return WIKIDOT_ROOT . '/web/files--sites/' . $p;
    }
}
