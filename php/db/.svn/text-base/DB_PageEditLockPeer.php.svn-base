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
class DB_PageEditLockPeer extends DB_PageEditLockPeerBase {

    public function deleteOutdated($pageId) {
        $c = new Criteria();
        $c->add("page_id", $pageId);
        $d = new ODate();
        $c->add("date_last_accessed", $d->addSeconds(-15 * 60), '<');
        $this->delete($c);
    }

    public function deleteOutdatedByPageName($siteId, $pageName) {
        $c = new Criteria();
        $c->add("page_unix_name", $pageName);
        $c->add("site_id", $siteId);
        $d = new ODate();
        $c->add("date_last_accessed", $d->addSeconds(-15 * 60), '<');
        $this->delete($c);
    }

}
