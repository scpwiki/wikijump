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
 * @package Wikidot_Web
 * @version $Id: lucene_search.php,v 1.1 2008/12/04 12:16:45 redbeard Exp $
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

class Wikidot_Search_Highlighter {
	
	static public function highlightIfSuitable($html, $request_uri, $referer) {
	
		if (self::suitable($request_uri) && $query = self::query($referer)) {
			
			$queryObj = Zend_Search_Lucene_Search_QueryParser::parse($query);
			$out = $queryObj->highlightMatches($html);
			
			if (! $out) {
				return $html;
			}
			
			$htmlNice = self::joinHtml($html, $out);
			
			if ($htmlNice) {
				return $htmlNice;
			}
		}
		
		return $html;
	}
	
	static protected function query($referer) {

		$host = parse_url($referer, PHP_URL_HOST);
		$path = parse_url($referer, PHP_URL_PATH);
		$query = parse_url($referer, PHP_URL_QUERY);
		
		$a = array();
		parse_str($query, $a);
		
		// Google search
		if ($path == '/search' && isset($a['q'])) {
			return $a['q'];
		}
		
		// Yahoo search
		if (preg_match('|^/search;|', $path) && isset($a['p'])) {
			return $a['p'];
		}
			
		// Wikidot search
		$a = array();
		if (preg_match(";/search:(site|all)/(a/[pf]*/)?q/([^/]*)($|/);", $path, $a)) {
			return $a[3];
		}
		
		return null;
	}
	
	// highlight is not suitable for the main page (/) and search pages themselves
	static protected function suitable($request_uri) {
		
		return ! preg_match(";^/($|search:);", $request_uri);
		
	}
	
	static protected function joinHtml($html, $out) {
		$dom = new DOMDocument();
		@$dom->loadHTML($out);
		
		$x = new DOMXPath($dom);
		$xa = $x->query('id("main-content")');
		$out_main = $xa->item(0);
		
		if (! $out_main) {
			return null;
		}
		
		$dom = new DOMDocument();
		@$dom->loadHTML($html);
		
		$x = new DOMXPath($dom);
		$xa = $x->query('//div[@id="main-content"]');
		$main = $xa->item(0);
		
		if (! $main) {
			return null;
		}
		
		$x = new DOMXPath($dom);
		$xa = $x->query('//div[@id="content-wrap"]');
		$wrapper = $xa->item(0);
		
		if (! $wrapper) {
			return null;
		}

		$out_main = $dom->importNode($out_main, true);
		$wrapper->replaceChild($out_main, $main);
		
		return $dom->saveHTML();
	}
}
