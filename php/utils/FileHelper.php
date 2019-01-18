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
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

class FileHelper {
	
	public static function totalSiteFilesSize($siteId){
		$q = "SELECT sum(size) AS size FROM file WHERE site_id='".db_escape_string($siteId)."'	";
		$db = Database::connection();
		$r = $db->query($q)->nextRow();
		return $r['size'];
	}
	
	public static function totalPageFilesSize($pageId){
		$q = "SELECT sum(size) as size FROM file WHERE page_id='".db_escape_string($pageId)."'	";
		$db = Database::connection();
		$r = $db->query($q)->nextRow();
		return $r['size'];
	}
	
	public function totalSiteFileNumber($siteId){
		$q = "SELECT count(*) AS count FROM file WHERE site_id='".db_escape_string($siteId)."'	";
		$db = Database::connection();
		$r = $db->query($q)->nextRow();
		return $r['count'];
	}
	
	public static function formatSize($size){
		$filesizename = array(" Bytes", " kB", " MB", " GB", " TB", " PB");
  	 	if($size == 0){
  	 		return "0 Bytes";
  	 	}
  	 	return round($size/pow(1024, ($i = floor(log($size, 1024)))), 2) . $filesizename[$i];	
	}
}
