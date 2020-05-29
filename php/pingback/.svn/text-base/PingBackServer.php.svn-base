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

require_once('Zend/XmlRpc/Server/Fault.php');

/* TODO: static? */
class PingBackServer {
	/**
	 * ping method for pingback xmlrpc server
	 *
	 * @param string $sourceURI The absolute URI of the post on the source page containing the link to the target site.
	 * @param string $targetURI The absolute URI of the target of the link, as given on the source page.
	 * @throws Zend_XmlRpc_Server_Fault on caugth exceptions
	 * @return string on success
	 */
	static public function ping($sourceURI, $targetURI) {
		
		$pb = new PingBack($sourceURI, $targetURI);
		
		Zend_XmlRpc_Server_Fault::attachFaultException("PingBackException");
		
		try {
			$ret = $pb->pong();
			
			// do something
			
		} catch(PingBackException $e) {
			throw new Zend_XmlRpc_Server_Fault($e);
		}
		
		return "OK";
	}
}
