<?php





/**
 * Action class.
 *
 */
abstract class Action {

	public function isAllowed($runData){
		return true;
	}
	public abstract function perform($runData);
}
