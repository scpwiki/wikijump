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
 * @version $Id: UploadedFileFlowController.php,v 1.9 2008/08/28 12:01:29 redbeard Exp $
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

class UploadedFileFlowController extends WikidotController {

	/**
	 * Restricted areas in wikis /local--*
	 * i.e. if files is in the array, /local--files is restricted
	 * otherwise files in /local--files/ are allways public (for private wikis too)
	 *
	 * @var unknown_type
	 */
	static protected $RESTRICTED_AREAS = array("resized-images", "files", "code", "auth");
	
	/**
	 * displays a forbidden screen and send 401 HTTP response code
	 *
	 */
	protected function forbidden() {
		header("HTTP/1.0 401 Unauthorized");
		$this->setContentTypeHeader("text/html");
		echo "Not authorized. This is a private site with access restricted to its members.";
	}

	/**
	 * Build URL from site name, domain and file name
	 *
	 * @param DB_Site $site site to get name from
	 * @param string $domain domain to use
	 * @param string $file file to redirect to
	 */
	protected function buildURL($site, $domain, $file) {

		$proto = ($_SERVER['HTTPS']) ? 'https' : 'http';
		$host = $site->getUnixName() . "." . $domain;

		$url = "${proto}://${host}/local--${file}";

		return $url;
	}

	/**
	 * checks whether file is from a public area (public wiki or non-restricted directory)
	 *
	 * @param DB_Site $site
	 * @param string $file
	 * @return boolean
	 */
	protected function publicArea($site, $file) {
		if (! $site) {
			return false;
		}

		if (! $site->getPrivate()) { // site is public
			return true;
		}

		$dir = array_shift(explode("/", $file));
		if (! in_array($dir, self::$RESTRICTED_AREAS)) {
			return true;
		}

		return false;
	}

	/**
	 * Checks if user is allowed to view a file
	 * 
	 * public because FilesAuthScriptModule needs it
	 *
	 * @param DB_OzoneUser $user
	 * @param DB_Site $site
	 * @param string $file
	 * @return bool
	 */
	public function userAllowed($user, $site, $file = "auth/") {

		if ($this->publicArea($site, $file)) {
			return true;
		}

		if (! $user) {
			return false;
		}

		if ($user->getSuperAdmin() || $user->getSuperModerator() || $this->member($user, $site)) {
			return true;
		}

		return false;
	}

	/**
	 * builds the path to local file
	 *
	 * @param DB_Site $site
	 * @param string $file
	 * @return string
	 */
	protected function buildPath($site, $file) {
		return $site->getLocalFilesPath().'/'.$file;
	}

	/**
	 * detects from "file" if this is code request
	 *
	 * @param string $file
	 * @return bool
	 */
	protected function isCodeRequest($file) {
		if (preg_match(";^code/;", $file)) {
			return true;
		} else {
			return false;
		}
	}

	/**
	 * detects from "file" if this is auth request
	 *
	 * @param string $file
	 * @return bool
	 */
	protected function isAuthRequest($file) {
		if (preg_match(";^auth/;", $file)) {
			return true;
		} else {
			return false;
		}
	}

	/**
	 * Serves a code extracted from the page
	 *
	 * @param DB_Site $site
	 * @param string $fileName code/pagename/number
	 * @param int $expires timeout in seconds
	 */
	protected function serveCode($site, $fileName, $expires = 0, $restrict_html = false) {
		$m = array();

		if (preg_match(";^code/([^/]+)/?(?:/([0-9]+))?(?:(/r/)(.*))?$;", $fileName, $m)) {
			$pageName = $m[1];
			$number = 1;
			if (isset($m[2])) {
				$number = (int) $m[2];
			}
			if (isset($m[3])) {
				$params = array();
				if (isset($m[4])) {
					parse_str($m[4], $params);
				}
			} else {
				$params = null;
			}
			
			$ext = new CodeblockExtractor($site, $pageName, $number, $params);
			
			$mime = $ext->getMimeType();
			if ($restrict_html && preg_match(self::$HTML_MIME_TYPES, $mime)) {
				$mime = self::$HTML_SERVE_AS;
			}
		
			$this->setContentTypeHeader($mime);
			$this->setExpiresHeader($expires);
			
			echo $ext->getContents();
				
		} else {
			$this->fileNotExists();
		}
	}

	/**
	 * Serves an auth response which is a redirect back to the supplied URL
	 *
	 * @param string $fileName auth/url
	 */
	protected function serveAuthResponse($fileName) {

		if (preg_match(";^auth/(.*)$;", $fileName, $m)) {
				
			$this->redirect(urldecode($m[1]));
		} else {
			$this->fileNotExists();
		}
	}

	public function process() {

		Ozone::init();

		$runData = new RunData();
		$runData->init();
		Ozone::setRunData($runData);

		$siteHost = $_SERVER['HTTP_HOST'];
		$site = $this->siteFromHost($siteHost, false, true);

		if (! $site) {
			$this->siteNotExists();
			return;
		}
		
		if ($site->getSettings()->getSslMode() == "ssl_only" && ! $_SERVER['HTTPS']) {
			header("HTTP/1.1 301 Moved Permanently");
			header("Location: https://$_SERVER[HTTP_HOST]$_SERVER[REQUEST_URI]");
			return;
		}

		$file = urldecode($_SERVER['QUERY_STRING']);
		$file = preg_replace("/\\?[0-9]+\$/", "", $file);
		$file = preg_replace("|^/*|", "", $file);

		if (! $file) {
			$this->fileNotExists();
			return;
		}

		$path = $this->buildPath($site, $file);

		if ($this->isUploadDomain($siteHost) || ! GlobalProperties::$USE_UPLOAD_DOMAIN) {
				
			if ($this->publicArea($site, $file)) {
					
				if ($this->isCodeRequest($file)) {
					$this->serveCode($site, $file, GlobalProperties::$CACHE_FILES_FOR, GlobalProperties::$RESTRICT_HTML);
				} else {
					$this->serveFileWithMime($path, GlobalProperties::$CACHE_FILES_FOR, GlobalProperties::$RESTRICT_HTML);
				}

				return;

			} else {
					
				/* NON PUBLIC AREA -- CHECK PERMISSION! */

				$runData->handleSessionStart();
				$user = $runData->getUser();

				if ($this->userAllowed($user, $site, $file)) {
						
					if ($this->isCodeRequest($file)) {
						$this->serveCode($site, $file, -3600);
					} elseif ($this->isAuthRequest($file)) {
						$this->serveAuthResponse($file);
					} else {
						$this->serveFileWithMime($path, -3600, GlobalProperties::$RESTRICT_HTML);
					}
					return;
						
				} else {
						
					$url = $this->buildURL($site, GlobalProperties::$URL_DOMAIN, $file);
					$this->redirect($url);
					return;
				}
			}
				
		} else {

			/* NOT UPLOAD DOMAIN, so it's *.wikidot.com or a custom domain */
				
			if ($this->publicArea($site, $file)) {

				$url = $this->buildURL($site, GlobalProperties::$URL_UPLOAD_DOMAIN, $file);
				$this->redirect($url);
				return;

			} else {

				$runData->handleSessionStart();
				$user = $runData->getUser();

				if ($this->userAllowed($user, $site, $file)) {
					
					$siteFilesDomain = $site->getUnixName() . "." . GlobalProperties::$URL_UPLOAD_DOMAIN;
					
					$skey = $runData->generateSessionDomainHash($siteFilesDomain);
					$user_id = $user->getUserId();
						
					$file_url = $this->buildURL($site, GlobalProperties::$URL_UPLOAD_DOMAIN, $file);
					$url = $siteFilesDomain . CustomDomainLoginFlowController::$controllerUrl;
						
					$this->redirect($url, array("user_id" => $user_id, "skey" => $skey, "url" => $file_url), true);
					return;
				}
			}
		}

		$this->forbidden();

	}
}
