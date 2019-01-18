#!/usr/bin/env php
<?php

/**
 * Wikidot (Community Edition) - free wiki collaboration software
 * 
 * 							http://www.wikidot.org
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
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 * 
 * @category Wikidot
 * @package Wikidot_Tools
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc. (http://www.wikidot-inc.com)
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

chdir(dirname(__FILE__)); // unifies CLI/CGI cwd handling
require ('../php/setup.php');

// connect to the database
Database::init();

$db = Database::connection();
$db->begin();

$files = $argv;
array_shift($files);

while (count($files)) {
    $dump = file_get_contents('../' . $files[0]);
    $query_no = 0;

    foreach (explode(';;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;', $dump) as $query) {
        try {
            $query_no++;
            if (trim($query) != "") {
                $db->query($query);
            }
        } catch (OzoneDatabaseException $e) {
            die("\n\nError occured at query number " . $query_no . ', file ' . $files[0] . ":\n" . htmlspecialchars($query) . "\n");
        }
    }

    array_shift($files);
}

$db->commit();

