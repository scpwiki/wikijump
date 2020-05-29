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
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

require_once('Zend/Http/Client.php');
require_once('Zend/XmlRpc/Client.php');
require_once('Zend/XmlRpc/Client/FaultException.php');
require_once('Zend/Http/Client/Adapter/Exception.php');
require_once('Zend/Http/Response.php');

/**
 * The Wikidot PingBack class.
 * 
 * Use it, to ping external services, and process other services ping requests
 * using PingBackServer as a frontend to this 
 *
 */
class PingBack {
	
	/**
	 * HTML block elements that can surround link and thus can be treated as a context of the link
	 *
	 * @var array
	 */
	private static $BLOCK_ELEMENTS = array("div", "p", "body", "ul", "ol", "td", "pre", "center");
	
	/**
	 * Content-types for HTML
	 * 
	 * @var array
	 */
	private static $HTML_CONTENT_TYPES = array('text/html', 'application/xhtml+xml', 'application/xml', 'text/xml');
	
	/**
	 * How much of page body to fetch (in bytes). We don't need
	 * the whole document, because the head is at the top.
	 * NOT USED YET!
	 * 
	 * @var int
	 */
	private static $HTML_FETCH_BYTES = 2048;
	
	/**
	 * how many bytes we want in context before and after the link
	 * the context is cut to full words anyways
	 *
	 * @var int
	 */
	private static $CONTEXT_BYTES = 200;
	
	/**
	 * Constructs the PingBack object.
	 *
	 * @throws PingBackException if the Wikidot URI is wrong
	 * @param string $externalURI external URI
	 * @param string $wikidotURI Wikidot URI
	 */
	public function __construct($externalURI, $wikidotURI) {
		
		if ($this->isValidWikidotURI($wikidotURI)) {
			$this->wikidotURI = $wikidotURI;
		} else {
			throw new PingBackException("The specified target URI cannot be used as a target", 33);	
		}
		$this->externalURI = $externalURI;
		
	}
	
	/**
	 * Pingback from Wikidot page (source URI) to external page (target URI)
	 *
	 * @throws PingBackException if pinging is not successfull
	 * @throws PingBackNotAvailableException when the target is not PingBack enabled
	 * @return string the endpoint return value
	 */
	public function ping() {
		try {
			
			$rpc = new Zend_XmlRpc_Client($this->getExternalPingBackURI());
			$srv = $rpc->getProxy('pingback');
			return $srv->ping($this->wikidotURI, $this->externalURI);
			
		} catch (Zend_Http_Client_Adapter_Exception $e) {
			throw new PingBackException("HTTP Error: " . $e->getMessage());
		} catch (Zend_Http_Client_Exception $e) {
			throw new PingBackException("HTTP Error: " . $e->getMessage());
		} catch (Zend_XmlRpc_Client_FaultException $e) {			
			throw new PingBackException("XMLRCP Error: " . $e->getMessage(), $e->getCode());
		} catch (PingBackException $e) {
			throw new PingBackException("Pingback Error: " . $e->getMessage());
		} catch (Exception $e) {
			throw new PingBackException("Unknown Error: " . $e->getMessage());
		}
	}
	
	/**
	 * Processes a pingback from external page (source URI) to Wikidot page (target URI)
	 * 
	 * Returns an array containing to keys: 'title' (with value of the page title)
	 * and 'context' which contains the context in which the link to Wikidot page appears
	 * 
	 * If some error apears the exception thrown should in most cases have the error code set
	 * Error codes for the PingBack are listed here: http://hixie.ch/specs/pingback/pingback#TOC3
	 * 
	 * @throws PingBackException in case of errors
	 * @return array array of title and the context of the link to Wikidot page
	 */
	public function pong() {
		$ret = array();
		
		$ret['title'] = $this->getExternalTitle();
		$ret['context'] = $this->getExternalContext();
		$ret['extrnalURI'] = $this->externalURI;
		$ret['wikidotURI'] = $this->wikidotURI;
		return $ret;
	}
	
	/**
	 * Wikidot URI. When pinging Wikidot, or pinging from a Wikidot site, this must be a Wikidot page URI 
	 *
	 * @var string
	 */
	private $wikidotURI = null;
	
	/**
	 * External URI. When pinging Wikidot, or pinging from a Wikidot site, this must be the other page URI 
	 *
	 * @var string
	 */
	private $externalURI = null;
	
	private function isHtmlByContentType($contentType) {
		foreach (self::$HTML_CONTENT_TYPES as $ct) {
			if (preg_match(";^$ct;", $contentType)) {
				return true;
			}
		}
		return false;
	}
	
	/**
	 * Gets external site's PingBack XMLRPC endpoint URI
	 * Checks for the X-Pingback header and if this fails,
	 * searches for <link rel="pingback"> in the HTML
	 *
	 * @throws PingBackNotAvailableException when pingback URI is not specified or cannot be read
	 * @return string
	 */
	private function getExternalPingBackURI() {
		$extPage = $this->getExternalPageHead();
		$pb_url = $extPage->getHeader("X-Pingback");
		
		try {
			if (! $pb_url) {
				$html = $this->getExternalPageDomAsSimpleXml();
				$pb_urlx = $this->xpath1($html, "//link[@rel='pingback'][1]");
				if (! $pb_urlx) {
					throw new Exception();
				}
				$pb_url = $pb_urlx["href"];
			}
			if (! $pb_url) {
				throw new Exception();
			}
		} catch (PingBackException $e) {
			throw new PingBackNotAvailableException("Site does not seem to support PingBack service; reason: " . $e->getMessage());
		} catch (Exception $e) {
			throw new PingBackNotAvailableException("Site does not seem to support PingBack service");
		}
		return (string) $pb_url;
	}
	
	/**
	 * Checks whether the supplied URI is a valid Wikidot URI
	 *
	 * @param string $uri
	 * @return bool true if the URI is a valid Wikidot URI
	 */
	private function isValidWikidotURI($uri) {
		/* TODO: validate */
		return true;
	}
	
	/**
	 * Fetches the title of the external page.
	 * If title is not set, the URI is returned
	 *
	 * @return string the HTML title or the URI of external page
	 */
	private function getExternalTitle() {
		$xml = $this->getExternalPageDomAsSimpleXml();
		
		try {
			
			$titles = $xml->xpath('//head/title');
			if (count($titles) < 1) {
				throw new Exception();
			}
			$title = $titles[0];
			if (empty($title)) {
				throw new Exception();
			}
			
		} catch (Exception $e) {
			$title = $this->externalURI();
		}
		
		return (string) $title;
	}
	
	/**
	 * Gets the context in which the link to Wikidot page appears on the external site
	 *
	 * @return string HTML with context of the page -- all tags are stripped, but the <a href> to us 
	 */
	private function getExternalContext() {
		
		$xml = $this->getExternalPageDomAsSimpleXml();
		
		$href = htmlspecialchars($this->wikidotURI);
		$path = "//body//a[@href=\"$href\"][1]";
		$link = $this->xpath1($xml, $path);
		
		if (! $link) {
			throw new PingBackException("The source URI does not contain a link to the target URI", 17);
		}
		
		$context = $link;
		
		// Searching for the smallest block element containing the link
		while (! in_array(strtolower($context->getName()), self::$BLOCK_ELEMENTS)) {
			$path .= "/..";
			$context = $this->xpath1($xml, $path);
		}
		
		// Expanding context
		$previous = $this->xpath1($xml, "$path/preceding-sibling::*[position()=1]");
		$next = $this->xpath1($xml, "$path/following-sibling::*[position()=1]");
		
		// Join this all
		$ret = "";
		if ($previous) {
			$ret = $previous->asXML();
		}
		$ret .= " " . $context->asXML() . " ";
		if ($next) {
			$ret .= " " . $next->asXML();
		}
		
		// Add more space
		$ret = preg_replace("|<([^/])|s", " <\\1", $ret);
		$ret = preg_replace("|</a>|s", "</a> ", $ret);
		
		// Strip tags but "a"
		$ret = strip_tags($ret, "<a>");
		
		// Sanitize "a" and add class delete
		$ret = preg_replace("|<a[^>]*href=\"([^\"]*)\"[^>]*>([^<]*)</a>|s", "<a class=\"delete\" href=\"\\1\">\\2</a>", $ret);
		
		// Find THE "a" tag and add a pingback class to it
		$xml = new SimpleXMLElement("<context>$ret</context>");
		$node = $this->xpath1($xml, "//a[@href=\"" . $href . "\"][1]");
		if ($node) {
			$node["class"] = "pingback";
		}
		$ret = strip_tags($xml->asXML(), "<a>");
		
		// Delete any "a" with class delete
		$ret = preg_replace("|<a[^>]*class=\"delete\"[^>]*>([^<]*)</a>|s", "\\1", $ret);
		
		// Fine cut the context
		$ret = preg_replace('|.*(.{' . self::$CONTEXT_BYTES . '}<a)|s', "\\1", $ret);
		$ret = preg_replace('|(a>.{' . self::$CONTEXT_BYTES . '}).*|s', "\\1", $ret);
		
		// Cut to words
		$ret = preg_replace("|^[^\\s]*\\s|s", "", $ret);
		$ret = preg_replace("|\\s[^\\s]*$|s", "", $ret);
		
		return $ret;
	}
	
	/**
	 * Queries SimpleXMLElement with XPath and return the first result or null if found nothing
	 *
	 * @param SimpleXMLElement $dom SimpleXMLElement object to query
	 * @param string $xpath the query
	 * @return SimpleXMLElement | null resulting element 
	 */
	private function xpath1($dom, $xpath) {
		$res = $dom->xpath($xpath);
		if (count($res) < 1) {
			return null;
		}
		return $res[0];
	}
	
	/**
	 * Requests the external page using the given method (mostly GET and HEAD)
	 *
	 * @param string $method
	 * @return Zend_Http_Response
	 */
	private function requestExternalPage($method = "GET") {
		try {
			$hc = new Zend_Http_Client($this->externalURI, array("strictredirects" => true));
			$resp = $hc->request($method);
			if ($resp->getStatus() != 200) {
				throw new PingBackException("Site does not exist", 16);
			}
		} catch (Zend_Http_Client_Adapter_Exception $e) {
			throw new PingBackException("HTTP error: " . $e->getMessage(), 16);
		}
		return $resp;
	}
	
	/**
	 * HTTP headers response object of the external URI
	 *
	 * @var Zend_Http_Response
	 */
	private $externalPageHead = null;
	
	/**
	 * Requests the HTTP header of the external URL unless already fetched
	 *
	 * @return Zend_Http_Response the HTTP response object
	 */
	private function getExternalPageHead() {
		if ($this->externalPageBody) {
			return $this->externalPageBody;
		}
		if (! $this->externalPageHead) {
			$this->externalPageHead = $this->requestExternalPage("HEAD");
		}
		return $this->externalPageHead;
	}
	
	/**
	 * HTTP (body and headers) response object of the external URI
	 *
	 * @var Zend_Http_Response
	 */
	private $externalPageBody = null;
	
	/**
	 * Requests the URL unless already fetched
	 *
	 * @return Zend_Http_Response the HTTP response object
	 */
	private function getExternalPageBody() {
		if (! $this->externalPageBody) {
			$this->externalPageBody = $this->requestExternalPage("GET");
		}
		return $this->externalPageBody;
	}
	
	/**
	 * SimpleXMLElement of the HTML from the external URI
	 *
	 * @var SimpleXMLElement
	 */
	private $externalPageDomAsSimpleXml = null;
	
	/**
	 * Gets the SimpleXMLElement of the HTML from the external URI
	 *
	 * @return SimpleXMLElement simple XML element of the external document
	 */
	private function getExternalPageDomAsSimpleXml() {
		
		if (! $this->externalPageDomAsSimpleXml) {
			$head = $this->getExternalPageHead();
			if ($this->isHtmlByContentType($head->getHeader("Content-type"))) {
			
				$html = $this->getExternalPageBody()->getBody();
				$dom = new DOMDocument();
				@$dom->loadHTML($html);
				
				$xml = @simplexml_import_dom($dom);
				if (is_a($xml, "SimpleXMLElement")) {
					$this->externalPageDomAsSimpleXml = $xml;
				}
				
			}
			
			if (! $this->externalPageDomAsSimpleXml) {
				throw new PingBackNotAvailableException("Cannot parse DOM - probably not a HTML response");
			}
		}
		
		return $this->externalPageDomAsSimpleXml;
	}
}
