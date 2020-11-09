<?php



/**
 * Logger tool.
 *
 */
class OzoneLogger {

	public static $LEVELS = array('fatal'=>0, 'error' => 1, 'warning' => 2, 'info' => 3, 'debug' => 4);

	public static $loggerInstance;

	public $loggerOutputs = array();
	public $debugLevel = 0;

	/** Returns the instance of Logger. If the instance does not exist -
	 * it is created.
	 */
	public static function instance(){
		if(self::$loggerInstance === null){
			self::$loggerInstance = new OzoneLogger();
		}
		return self::$loggerInstance;
	}

	/**
	 * Adds output for the logger. OzoneLogger object can have several outputs, e.g.
	 * file, database, html...
	 * @param OzoneLoggerOutput $loggerOutput
	 */
	public function addLoggerOutput($loggerOutput){
		$this->loggerOutputs[] = $loggerOutput;
	}

	/**
	 * Sets the debug level for the logger. Possible levels are: "error", "warning",
	 * "info", "debug" in the stated order. The "debug" level provides the most ferbose
	 * output that includes all error, warning, info and debug messages. "Info" level provides
	 * output only for error, warning and info. Get it? ;-)
	 * @param string $debugLevel
	 */
	public function setDebugLevel($debugLevel){
		$this->debugLevel = OzoneLogger::$LEVELS["$debugLevel"];
	}

	public function fatal($message){
		$this->runLogEvent(0, $message);
	}

	public function error($message){
		$this->runLogEvent(1, $message);
	}

	public function warning($message){
		$this->runLogEvent(2, $message);
	}

	public function info($message){
		$this->runLogEvent(3, $message);
	}

	public function debug($message){
		$this->runLogEvent(4, $message);
	}

	/**
	 * Processes the log event.
	 */
	private function runLogEvent($level, $message){
		// check if message has appropriate level to be processed
		if($level<=$this->debugLevel){
			// create event
			$backtrace = debug_backtrace();
			$event = array();
			// now some tricking-around
			$event['timestamp'] = time();
			$event['class'] = $backtrace[2]['class'];
			$event['function'] = $backtrace[2]['function'];
			$event['type'] = $backtrace[2]['type'];
			$event['line'] = $backtrace[1]['line'];
			$event['level'] = $level;
			$event['message'] = $message;

			foreach ($this->loggerOutputs as $output){
				$output->handleEvent($event);
			}
		}
	}
}
