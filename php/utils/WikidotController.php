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
 * @version $Id: UploadedFileFlowController.php,v 1.5 2008/08/01 14:00:27 quake Exp $
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

abstract class WikidotController extends WebFlowController {
	
	static protected $HTML_MIME_TYPES = array("text/html", "application/xhtml+xml", "application/xml", "text/xml");
	static protected $HTML_SERVE_AS = "text/plain";
	
	/**
	 * Gets a site from given hostname. This version works for custom domains and upload domain if needed
	 *
	 * @param string $siteHost the host to check
	 * @param bool $customDomains whether to check custom domains 
	 * @param bool $uploadDomain whether to check upload domains as well
	 * @return DB_Site
	 */
	protected function siteFromHost($siteHost, $customDomains = false, $uploadDomain = false) {
		
		$memcache = Ozone::$memcache;
		
		if ($uploadDomain) {
			$regexp = "/^([a-zA-Z0-9\-]+)\.(" . GlobalProperties::$URL_DOMAIN_PREG . "|" . GlobalProperties::$URL_UPLOAD_DOMAIN_PREG . ")$/";
		} else {
			$regexp = "/^([a-zA-Z0-9\-]+)\.(" . GlobalProperties::$URL_DOMAIN_PREG . ")$/";
		}
		
		if (preg_match($regexp, $siteHost, $matches) == 1) {
			// select site based on the unix name
				
			$siteUnixName = $matches[1];
			$mcKey = 'site..'.$siteUnixName;
			$site = $memcache->get($mcKey);
			if($site == false){
				$c = new Criteria();
				$c->add("unix_name", $siteUnixName);
				$c->add("site.deleted", false);
				$site = DB_SitePeer::instance()->selectOne($c);
				if($site) {
					$memcache->set($mcKey, $site, 0, 3600);
				}
			}
		}
		
		
		// select site based on the custom domain

		if (! $site && $customDomains) {
			$mcKey = 'site_cd..'.$siteHost;
			$site = $memcache->get($mcKey);
			if ($site == false) {	
				$c = new Criteria();
				$c->add("custom_domain", $siteHost);
				$c->add("site.deleted", false);
				$site = DB_SitePeer::instance()->selectOne($c);
				if ($site) {
					$memcache->set($mcKey, $site, 0, 3600);
				}	
			}
		}
		
		return $site;
	}
	
	protected function isUploadDomain($siteHost) {

		if (preg_match("/^[^.]*\." . GlobalProperties::$URL_UPLOAD_DOMAIN_PREG . "$/", $siteHost)) {
			return true;
		}

		return false;

	}

	protected function siteNotExists() {
		$this->serveFile(WIKIDOT_ROOT."/files/site_not_exists.html", "text/html");
	}

	protected function isBuggyIeDamnYouBastard(){
		if (isset($_SERVER['HTTP_USER_AGENT']) && strpos($_SERVER['HTTP_USER_AGENT'], 'MSIE') !== false){
			return true;
		} else {
			return false;
		}
	}
	
	protected function fileNotExists() {
		$this->serveFile(WIKIDOT_ROOT."/files/file_not_exists.html", "text/html");
	}
	
	private function calculateEtag($path) {
		if (file_exists($path)) {
			return '"' . md5_file($path) . '"';
		} else {
			return '"none"';
		}
	}
	
	public function return304() {
		header("HTTP/1.0 304 Not Modified");
	}

	/**
	 * serves a file of given path with autodetected MIME type and given expires (if any)
	 *
	 * @param string $path
	 * @param int $expires time in seconds
	 */
	protected function serveFileWithMime($path, $expires = null, $restrictHtml = false) {
		$etag = $this->calculateEtag($path);
		
		if (isset($_SERVER["HTTP_IF_NONE_MATCH"])) {
			if ($_SERVER["HTTP_IF_NONE_MATCH"] == $etag) {
				$this->return304();
				return;
			}
		}

		/* guess/set the mime type for the file */
		if ($dir == "theme" || preg_match("/\.css$/", $path)) {
			$mime = "text/css";
		} else if (preg_match("/\.js$/", $path)) {
			$mime = "text/javascript";
		}

		if (! isset($mime)) {
			$mime = $this->fileMime($path, $restrictHtml);
		}

		$this->serveFile($path, $mime, $expires, $etag);
	}

	/**
	 * checks if the user is a member of a site
	 *
	 * @param DB_OzoneUser $user
	 * @param DB_Site $site
	 * @return boolean
	 */
	protected function member($user, $site) {
		if (! $site || ! $user) {
			return false;
		}

		$c = new Criteria();
		$c->add("site_id", $site->getSiteId());
		$c->add("user_id", $user->getUserId());

		if (DB_MemberPeer::instance()->selectOne($c)) { // user is a member of the wiki
			return true;
		}

		return false;
	}

	/**
	 * detects MIME type of a file. Includes workarounds for buggy detection
	 *
	 * @param string $path path to file
	 * @return string the MIME type
	 */
	protected function fileMime($path, $restrictHtml = false) {

		if (file_exists($path)) {
			$mime =  FileMime::mime($path);
		} else {
			$mime = false;
		}
			
		if (! $mime || $mime == "application/msword") {
			$mime = "application/octet-stream";
		}
		
		if ($restrictHtml && in_array($mime, self::$HTML_MIME_TYPES)) {
			$mime = self::$HTML_SERVE_AS;
		}

		return $mime;
	}
}
