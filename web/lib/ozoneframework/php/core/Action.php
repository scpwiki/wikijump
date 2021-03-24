<?php

namespace Ozone\Framework;





/**
 * Action Class.
 *
 */
abstract class Action {

	public function isAllowed($runData){
		return true;
	}
	public abstract function perform($runData);
}
