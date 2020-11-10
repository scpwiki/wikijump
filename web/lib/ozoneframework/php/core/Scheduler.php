<?php


/**
 * Scheduler.
 *
 */
class Scheduler {

	private $jobs = array();

	/**
	 * Default path for job classes.
	 */
	private $classPath;

	/**
	 * Stores next execution times (as values) and job numbers (as keys).
	 */
	private $queue = array();

	private $lastJobRun;

	public function setClassPath($cp){
		$this->classPath = $cp;
	}

	public function getClassPath(){
		return $this->classPath;
	}

	public function addJob($job){
		$this->jobs[] = $job;
	}

	public function start(){
		foreach($this->jobs as $job){
			$job->prepare();
		}

		// initial queue
		$currentTime = new Date(); //currentDateUTC();

		$currentTime->addSeconds(2); // for safety.
		foreach($this->jobs as $jobKey => $job){
			$nextRunTime = $job->getNextRunTime($currentTime);
			echo $job->getName()." ".$nextRunTime->getDate()."\n";

			// set an entry in the queue
			$timestamp = $nextRunTime->getTime();
			$this->queue[$jobKey] = $timestamp;
		}

		asort($this->queue);

		// the main loop for execution...
		while(true){
			$this->tick();
		}

	}

	private function tick(){
		// get first element from the queue
		$nextRunTime  = reset($this->queue); // (hope this exists...)
		$nextJob = key($this->queue);
		// now calculate how much we should wait...
		$ct = new Date();

		$ct = (float) $ct->getTime();

		$nr = (float)$nextRunTime;

		$timeToWait = $nr - $ct;

		// sleep for no more than 1 minute
		if($timeToWait > 60){
			sleep(60);
			return;
		}

		$usecToSleep = (int) ($timeToWait*1000000.0);
		if($usecToSleep>0){
			usleep($usecToSleep);
		}

		// execute job
		$currentTime = new Date();

		$jobEntry = $this->jobs[$nextJob];
		echo $currentTime->getDate()." running job ".$jobEntry->getName()."\n";
		$jobEntry->getJobObject()->run();

		// update next run time
		unset($this->queue[$nextRunTime]);

		$currentTime->addSeconds(1);
		$nextRunTime = $jobEntry->getNextRunTime($currentTime);
		$timestamp = $nextRunTime->getTime();
		$this->queue[$nextJob] = $timestamp;
		asort($this->queue);
		$this->lastJobRun = $nextJob;
	}

	public function parseConfigXml($xml){
		foreach ($xml->job as $job) {
			$jobName = $job['name']."";

			$jobEntry = new JobEntry();

			$jobEntry->setName($jobName);
			$schedule = $job->time;

			// read and set "second"
			$second =  $schedule['second'];
			if($second !== null){
				$jobEntry->setSecond($second);
			}

			// read and set "minute"
			$minute =  $schedule['minute'];
			if($minute !== null){
				$jobEntry->setMinute($minute);
			}

			// read and set "hour"
			$hour =  $schedule['hour'];
			if($hour !== null){
				$jobEntry->setHour($hour);
			}

			// read and set "day"
			$day =  $schedule['day'];
			if($day !== null){
				$jobEntry->setDay($day);
			}

			// read and set "month"
			$month =  $schedule['month'];
			if($month !== null){
				$jobEntry->setMonth($month);
			}

			// read and set "dayOfWeek"
			$dayOfWeek =  $schedule['dayOfWeek'];
			if($dayOfWeek !== null){
				$jobEntry->setDayOfWeek($dayOfWeek);
			}

			// load class
			$classFile = $this->classPath."/".$jobName.".php";
			require_once($classFile);
			$object = new $jobName();
			$jobEntry->setJobObject($object);

			$this->addJob($jobEntry);
		}
	}

}
