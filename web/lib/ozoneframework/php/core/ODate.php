<?php





/**
 * Simple date class.
 */
class ODate {
	private $timestamp;

	public function __construct($timestamp=null){
		if($timestamp === null){
			$timestamp=time();
		} else {
			if(!is_numeric($timestamp)){
				// not a timestamp
				$timestamp = strtotime($timestamp);
			}
		}
		$this->timestamp = $timestamp;
	}

	public function getTimestamp(){
		return $this->timestamp;
	}

	public function setTimestamp($timestamp){
		$this->timestamp = $timestamp;
	}

	public function convertToTZ($diff, $dst=true){
		if($dst){
			$diff += date('I', $this->timestamp);
		}
		$this->timestamp -= $diff*60*60;
	}

	public function getDateArray(){
		return getdate($this->timestamp);
	}

	public function getDate($format=null){
		if($format === null){
			$format = 'c';
		}
		return date($format, $this->timestamp);
	}

	public function addSeconds($sec){
		$this->timestamp+=$sec;
		return $this;
	}

	public function subtractSeconds($sec){
		$this->timestamp-=$sec;
		return $this;
	}

	public function before($date){
		if($this->timestamp < $date->getTimestamp()){
			return true;
		} else {
			return false;
		}
	}

	public function after($date){
		if($this->timestamp > $date->getTimestamp()){
			return true;
		} else {
			return false;
		}
	}

	public function equals($date){
		if($this->timestamp === $date->getTimestamp()){
			return true;
		} else {
			return false;
		}
	}

	public function format($format){
		return strftime($format, $this->timestamp);
	}

}
