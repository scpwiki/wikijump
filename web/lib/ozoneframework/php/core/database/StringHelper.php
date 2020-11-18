<?php





/**
 * String utility class.
 *
 */
class StringHelper {

	public function propertyNameFirstCapitalized($property){
		return capitalizeFirstLetter(underscoreToLowerCase($property));
	}

}
