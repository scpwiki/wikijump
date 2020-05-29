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
class DB_PrivateMessage extends DB_PrivateMessageBase {

    public function getFromUser() {
        return DB_OzoneUserPeer::instance()->selectByPrimaryKey($this->getFromUserId());
    }

    public function getToUser() {
        if ($this->getToUserId() !== null) {
            return DB_OzoneUserPeer::instance()->selectByPrimaryKey($this->getToUserId());
        }
    }

    public function getPreview($length = 200) {
        
        $text = $this->getBody();
        
        $stripped = strip_tags($text);
        
        $substr = substr($stripped, 0, $length);
        if (strlen8($substr) == $length) {
            $substr = preg_replace('/\w+$/', "", $substr) . '...';
        }
        return $substr;
    }

}
