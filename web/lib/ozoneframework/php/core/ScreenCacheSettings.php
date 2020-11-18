<?php





/**
 * Settings for the screen cache.
 *
 */
class ScreenCacheSettings {

	protected $anonymousLayoutTimeout;
	protected $loggedLayoutTimeout ;
	protected $anonymousScreenTimeout;
	protected $loggedScreenTimeout;

	public function getLayoutTimeout($runData){
		if($runData->isUserAuthenticated()){
			$timeout = $this->loggedLayoutTimeout;
		} else {
			$timeout = $this->anonymousLayoutTimeout;
		}
		return $timeout;
	}

	public function isLayoutCacheable($runData){
		$timeout = 	$this->getLayoutTimeout($runData);
		if($timeout == null || $timeout == 0){
			return false;
		} else {
			return true;
		}
	}

	public function getScreenTimeout($runData){
		if($runData->isUserAuthenticated()){
			$timeout = $this->loggedScreenTimeout;
		} else {
			$timeout = $this->anonymousScreenTimeout;
		}
		return $timeout;
	}

	public function isScreenCacheable($runData){
		$timeout = 	$this->getScreenTimeout($runData);
		if($timeout == null || $timeout == 0){
			return false;
		} else {
			return true;
		}
	}

	public function setAnonymousLayoutTimeout($time){
		$this->anonymousLayoutTimeout = $time;
	}

	public function setAnonymousScreenTimeout($time){
		$this->anonymousScreenTimeout = $time;
	}

	public function setLoggedLayoutTimeout($time){
		$this->loggedLayoutTimeout = $time;
	}

	public function setLoggedScreenTimeout($time){
		$this->loggedScreenTimeout = $time;
	}

}
