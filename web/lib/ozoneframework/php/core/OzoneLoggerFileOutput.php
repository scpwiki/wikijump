<?php

namespace Ozone\Framework;



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
			$event['Class'], $event['type'], $event['function'],
			$event['line'],
			$event['message']);
		// quickly open file.
		$file = fopen($this->logFileName, "a");
		fwrite($file, $out);
		fclose($file);
		// how to make this atomic? synchronized?
	}

}
