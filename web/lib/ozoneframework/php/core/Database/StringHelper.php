<?php

namespace Ozone\Framework\Database;





/**
 * String utility Class.
 *
 */
class StringHelper {

	public function propertyNameFirstCapitalized($property){
		return capitalizeFirstLetter(underscoreToLowerCase($property));
	}

}
