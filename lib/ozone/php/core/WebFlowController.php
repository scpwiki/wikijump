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
 * @category Ozone
 * @package Ozone_Web
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */
 
/**
 * Web flow controller - abstract class.
 */
abstract class WebFlowController {
	abstract public function process();

	/**
	 * sets the Expires header
	 *
	 * @param int $expires time in seconds
	 */
	protected function setExpiresHeader($expires) {
		$expires = (int) $expires;

		if($this->isBuggyIeDamnYouBastard()){
			/*
			 * Sorry, no headers for Explorer. See this URL:
			 * http://www.alagad.com/go/blog-entry/error-internet-explorer-cannot-download-filename-from-webserver
			 */
			return;
		}
		if ($expires) {
			if ($expires > 0) {
				$date = gmdate("D, d M Y H:i:s", time() + $expires) . " GMT";
				header("Expires: " . $date);
			} else {
				header('Cache-Control: no-store, no-cache, must-revalidate');
				header('Cache-Control: post-check=0, pre-check=0', FALSE);
				header('Pragma: no-cache');
				header("Expires: Mon, 26 Jul 1997 05:00:00 GMT");
			}
		}
	}

	/**
	 * sets the Content-type header
	 *
	 * @param string $mime
	 */
	protected function setContentTypeHeader($mime) {
		if ($mime) {
			header("Content-type: $mime; charset=utf-8");
		}
	}

	/**
	 * sets the Content-type header
	 *
	 * @param string $mime
	 */
	protected function setEtagHeader($etag) {
		if ($etag) {
			header("Etag: $etag");
		}
	}
	
	/**
	 * Redirects browser to certain URL build from URL and params
	 *
	 * @param string $url URL to redirect to
	 * @param array $params params to pass with GET
	 * @param bool $addProtocol whether to add autodiscovered protocol to the front of URL
	 */
	protected function redirect($url, $params = null, $addProtocol = false) {
		
		if ($addProtocol) {
			$proto = ($_SERVER["HTTPS"]) ? "https" : "http";
			$url = "$proto://$url";
		}
		
		if (is_array($params)) {
			$url = $url . "?" . http_build_query($params);
		}
		
		header('HTTP/1.1 301 Moved Permanently');
		header("Location: $url");
		
	}

	/**
	 * serves the file using file path, mime type and expire offset
	 *
	 * @param string $path the file to serve
	 * @param string $mime the mime to set
	 * @param int $expires time in seconds to expire
	 */
	protected function serveFile($path, $mime = null, $expires = null, $etag = null) {
		if (file_exists($path)) {
			$this->setContentTypeHeader($mime);
			$this->setExpiresHeader($expires);
			$this->setEtagHeader($etag);
			$this->readfile($path);
		} else {
			$this->setContentTypeHeader("text/html");
			$this->readfile(WIKIDOT_ROOT."/files/file_not_exists.html");
		}
	}

	/**
	 * sends the file to the browser using PHP's readfile or X-Sendfile header
	 *
	 * @param unknown_type $path
	 */
	protected function readfile($path) {
		if (GlobalProperties::$XSENDFILE_USE) {
			header(GlobalProperties::$XSENDFILE_HEADER . ": $path");
		} else {
			readfile($path);
		}
	}
}
