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
 * @package Ozone_Logger
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

/**
 * File output for logger.
 *
 */
class OzoneLoggerFileOutput implements OzoneLoggerOutput {
	
	private $logFileName;
	
	/** 
	 * Sets output file name for logging.
	 */
	public function setLogFileName($fileName){
		$this->logFileName = $fileName;	
	}
	
	public function handleEvent($event){
		// first create the output string:
		$debugLevelString = array_search($event['level'], OzoneLogger::$LEVELS);
		$out = sprintf("[%s] %s, %s%s%s, line %d:  %s\n", $debugLevelString,
			date("Y.m.d G:i:s T", $event['timestamp']),
			$event['class'], $event['type'], $event['function'],
			$event['line'],
			$event['message']); 
		// quickly open file. 
		$file = fopen($this->logFileName, "a");
		fwrite($file, $out);
		fclose($file);
		// how to make this atomic? synchronized?
	}

}
