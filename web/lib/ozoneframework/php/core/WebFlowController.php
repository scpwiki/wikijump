<?php

namespace Ozone\Framework;





use Wikidot\Utils\GlobalProperties;

/**
 * Web flow controller - abstract Class.
 */
abstract class WebFlowController {
	abstract public function process();

	/**
	 * Sets cross-origin headers for improved security.
	 *
	 * See https://scuttle.atlassian.net/browse/WJ-452
	 */
	protected function setCrossOriginHeaders() {
		header("Cross-Origin-Opener-Policy: same-origin");
		header("Cross-Origin-Embedder-Policy: require-corp");
	}

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
	protected function setContentTypeHeader($mime = false) {
		if ($mime) {
			header("Content-type: $mime; charset=utf-8");
		}
	}

	/**
	 * sets the Etag header
	 *
	 * @param string $etag
	 */
	protected function setEtagHeader($etag = false) {
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

		if ($addProtocol == true) {
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
		$this->setCrossOriginHeaders();

		if (file_exists($path)) {
			$this->setContentTypeHeader($mime);
			$this->setExpiresHeader($expires);
			$this->setEtagHeader($etag);
			$this->readfile($path);
		} else {
			$this->setContentTypeHeader("text/html");
			$this->readfile(WIKIJUMP_ROOT."/files/file_not_exists.html");
		}
	}

	/**
	 * sends the file to the browser using PHP's readfile or X-Sendfile header
	 *
	 * @param mixed $path
	 */
	protected function readfile($path) {
		if (GlobalProperties::$XSENDFILE_USE) {
			header(GlobalProperties::$XSENDFILE_HEADER . ": $path");
		} else {
			readfile($path);
		}
	}
}
