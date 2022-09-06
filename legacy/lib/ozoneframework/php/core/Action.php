<?php

namespace Ozone\Framework;

/**
 * Action Class. Wowee.
 */
abstract class Action {

	public function isAllowed($runData){
		return true;
	}
	abstract public function perform($runData);
}
