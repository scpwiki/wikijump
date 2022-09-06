<?php

namespace Ozone\Framework;





/**
 * Abstract Class for screens.
 */
abstract class Screen {

	public function isAllowed($runData){
		return true;
	}

	abstract function render($runData);
}
