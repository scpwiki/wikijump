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
 * @package Ozone_Cron
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

/**
 * Job entry for the cron.
 *
 */
class JobEntry {
	
	private $name;
	private $jobObject;
	
	private $second = "0";
	private $minute = "0";
	private $day = "*";
	private $month = "*";
	private $dayOfWeek = "*";
	
	private $secondArray;
	private $minuteArray;
	private $dayArray;
	private $monthArray;
	private $dayOfWeekArray;
	
	public function setName($name){
		$this->name=$name;	
	}
	
	public function getName(){
		return $this->name;	
	}
	
	public function setJobObject($object){
		$this->jobObject = $object;	
	}
	
	public function getJobObject(){
		return $this->jobObject;	
	}
	
	public function getNextRunTime($date){
		// check if $date is a timestamp...
		if(($date instanceof Date) == false && is_integer($date) ){
			$date = new Date($date);			
		}
		// assume now $date IS instanceof Date
		
		$cSecond = $date->getSecond();
		$cMinute = $date->getMinute();
		$cHour = $date->getHour();
		$cDay = $date->getDay();
		$cMonth = $date->getMonth();
		$cYear = $date->getYear(); // required to check the number of days in the month

		$found = false;
		while($found === false){
			
			while($found === false){
				// iterate months...
				$cMonth = $this->findNextInArray($cMonth, $this->monthArray);
				if($cMonth === null){
					break;	
				}
				// find the day now
				while($found === false){
					$cDay = $this->findNextInArray($cDay, $this->dayArray);
					if($cDay === null){
						break;	
					}
					// here dayOfWeek and number of days in month should be checked!
					$date = new Date();
					$date->setYear($cYear);
					$date->setMonth($cMonth);
					$numberOfDaysInMonth = $date->getDaysInMonth();
					if($cDay>$numberOfDaysInMonth){
						break;	
					}
					if($this->dayOfWeekArray !== null){
						// get day of the week
						$date->setDay($cDay);
						$dayOfWeek = $date->getDayOfWeek();
						if(!in_array($dayOfWeek, $this->dayOfWeekArray)){
							$cDay++;
							continue;	
						}	
					}
					
					while($found === false){
						if($cHour == 24){break;}
						$cHour = $this->findNextInArray($cHour, $this->hourArray);
						if($cHour === null){
							break;	
						}
						while($found === false){
							if($cMinute == 60){	break;}
							$cMinute = 	$this->findNextInArray($cMinute, $this->minuteArray);
							if($cMinute === null){
								break;	
							}
							while($found === false){	
								if($cSecond==60){break;}					
								$cSecond = 	$this->findNextInArray($cSecond, $this->secondArray);
								if($cSecond === null){
									break;		
								}else{
								
									// FOUND IT!!! WOOOO!
									// create Date object	
									$date = new Date();
									$date->setYear($cYear);
									$date->setMonth($cMonth);
									$date->setDay($cDay);
									$date->setHour($cHour);
									$date->setMinute($cMinute);
									$date->setSecond($cSecond);
									
									return $date;
									
								}
							}
							$cMinute++;
							$cSecond = 0;
							
						}
						$cHour++;
						$cMinute = 0;
						$cSecond = 0;
						
					}
					$cDay++;
					$cHour = 0;
					$cMinute = 0;
					$cSecond = 0;
				}
				$cMonth++;	
				$cDay=0;
				$cHour = 0;
				$cMinute = 0;
				$cSecond = 0;
			}
			$cYear++; 
			$cMonth = 0;
			$cDay=0;
			$cHour = 0;
			$cMinute = 0;
			$cSecond = 0;
		}
	}
	
	public function setSecond($second){
		$this->second = $second;	
	}
	
	public function setMinute($minute){
		$this->minute = $minute;	
	}
	
	public function setHour($hour){
		$this->hour = $hour;	
	}
	
	public function setDay($day){
		$this->day = $day;	
	}
	
	public function setMonth($month){
		$this->month = $month;	
	}
	public function setDayOfWeek($dayOfWeek){
		$this->dayOfWeek = $dayOfWeek;	
	}

	public function prepare(){
		$this->updateScheduleArray();
		// load object
	}

	private function updateScheduleArray(){
		// update seconds first...
		$this->secondArray = $this->intervalStringToArray($this->second, 0, 59);
		$this->minuteArray = $this->intervalStringToArray($this->minute, 0, 59);
		$this->hourArray = $this->intervalStringToArray($this->hour, 0, 23);
		$this->dayArray = $this->intervalStringToArray($this->day, 1, 31); // assume 31 but a condition check is required later
		$this->monthArray = $this->intervalStringToArray($this->month, 1, 12);
		$this->dayOfWeekArray = $this->intervalStringToArray($this->dayOfWeek, 0, 6);
	}
	
	/**
	 * Given the interval devinition string for a given property and allowed
	 * range the method returns all allowed values as an array.
	 */
	private function intervalStringToArray($string, $rangeMin, $rangeMax){
		
		// if "*"
		if($string === "*"){
			$result = null;
			return $result;	
		}
		
		//if $string is just an integer...
		if(eregi("^[0-9]+$",$string)){
			$result = array();
			$result[]=(int)$string;
			return $result;	
		}

		// if is a coma-separated list:
		if(eregi("^[0-9]+(,[0-9]+)+$",$string)){
			$result = array();
			$result=explode(",",$string);
			for($i = 0; $i<count($result); $i++){
				$result[$i] = (int)$result[$i];	
			}	
			return $result;	
		}
		
		// if of form "*/n"
		if(eregi("^\*/[0-9]+$",$string)){
			$result = array();
			$repeat = (int)substr($string, 2);
			for($i=0;$i<=$rangeMax; $i+=$repeat){
				$result[]=$i;	
			}
			return $result;	
		}
		
		return null;
	}
	
	/**
	 * Finds next values in a sorted array that is equal or greater than the starting
	 * value. If not found - null is returned.
	 */
	private function findNextInArray($startval, $array){
		if($array === null){
			return $startval; // null means "any" here. so the starting value is just fine.	
		}
		
		$count = count($array);
		for($i = 0; $i<$count; $i++){
			if($array[$i] >= $startval){
				return $array[$i];	
			}	
		}
		
		return null;
			
	}
}
