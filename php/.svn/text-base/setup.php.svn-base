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

if(!defined('WIKIDOT_SETUP_COMPLETED')){
    
    // assume that computer's clock runs in UTC
    putenv("TZ=UTC");
    if(function_exists('date_default_timezone_set')){
        date_default_timezone_set('UTC');
    }
    
	// add settings for error-reporting
	error_reporting(E_ALL&~E_NOTICE); // hardcode ;-)
	
	// determine WIKIDOT_ROOT directory
	if(!defined('WIKIDOT_ROOT')){
		define('WIKIDOT_ROOT', dirname(dirname(__FILE__)));
		define('OZONE_ROOT', WIKIDOT_ROOT.DIRECTORY_SEPARATOR.'lib'.DIRECTORY_SEPARATOR.'ozone');
	}
	
	require_once (WIKIDOT_ROOT.DIRECTORY_SEPARATOR."php/utils/GlobalProperties.php");
	require_once (WIKIDOT_ROOT.DIRECTORY_SEPARATOR."lib/ozone/php/core/functions.php");
	require_once (WIKIDOT_ROOT.DIRECTORY_SEPARATOR."lib/ozone/php/core/autoload.inc.php");
	
	define('WIKIDOT_SETUP_COMPLETED', true);
}
