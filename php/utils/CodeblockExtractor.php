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

class CodeblockExtractor {

	protected $mimeType = null;
	protected $contents = "";
	
	protected $mimeMap = array(
		"css"	=> "text/css",
		"html"	=> "text/html",
	);
	
	public function __construct($site, $pageName, $codeblockNo = 1){
		try {
			$codeblockNo = (int) $codeblockNo;
			if ($codeblockNo < 1) {
				$codeblockNo = 1;
			}
			
			/* Check the cache. */
			
			$mkey = 'pagecodeblocks..' . $site->getSiteId() . '..' . $pageName;
			$m = OZONE::$memcache;
			$allMatches = $m->get($mkey);
			if(!$allMatches) {
				
				$page = DB_PagePeer::instance()->selectByName($site->getSiteId(), $pageName);
				
				if($page == null){
					throw new ProcessException("No such page");
				}
				// page exists!!! wooo!!!
				
				$source = $page->getSource();
				/* Get code block. */
				
				$regex = ';^\[\[code(\s[^\]]*)?\]\]((?:(?R)|.)*?)\[\[/code\]\](\s|$);msi';
				
				$allMatches = array();
				preg_match_all($regex, $source, $allMatches);
				
				$m->set($mkey, $allMatches, 0, 3600);
			}
			
			if(count($allMatches[2]) < $codeblockNo ) {
				throw new ProcessException('No valid codeblock found.');
			}
			
			$code = $allMatches[2][$codeblockNo - 1];
			if ($allMatches[1][$codeblockNo - 1]) {
				$params = $allMatches[1][$codeblockNo - 1];
				$m = array();
				$type = null;
				if (preg_match(':type="([^"]+)":', $params, $m)) {
					$type = strtolower($m[1]);
				}
				if (array_key_exists($type, $this->mimeMap)) {
					$this->mimeType = $this->mimeMap[$type];
				}
			}
			
			$this->contents = trim($code)."\n";
			
		} catch(Exception $e) {
			$this->contents = $e->getMessage();
		}
	}
	
	public function getContents() {
		return $this->contents;
	}
	
	public function getMimeType() {
		if ($this->mimeType) {
			return $this->mimeType;
		}
		return "text/plain";
	}

}
