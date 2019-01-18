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
 * @package Ozone_Web
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

/**
 * Implements a set of default rules for form validation. You can easily extend
 * this class and write own rules following the scheme for
 * creating new methods: <rulename>Rule($fieldValue, $ruleValue).
 */
class BaseFormValidator {
	
	/**
	 * Checks if the field is an integer value.
	 */
	public function integerRule($fieldValue, $ruleValue=null){
		if(ereg("^(\-)?[0-9]+$", $fieldValue)){
			return true;
		} else {
			return false;
		}
	}
	
	/**
	 * Checks if value as greater than the minimal value given.
	 */
	public function minvalueRule($fieldValue, $ruleValue){
		if($fieldValue < $ruleValue){
			return false;
		} else{
			return true;
		}
	}
	
	/**
	 * Checks if value as greater than the maximal value given.
	 */
	public function maxvalueRule($fieldValue, $ruleValue){
		if($fieldValue > $ruleValue){
			return false;
		} else{
			return true;
		}
	}
	
	/**
	 * Check if the string is shorter than the maximum length given.
	 */
	public function maxlengthRule($fieldValue, $ruleValue){
		if(strlen($fieldValue)>$ruleValue){
			return false;
		} else {
			return true;	
		}		
	}
	
	/**
	 * Check if the string is longer than the minimum length given.
	 */
	public function minlengthRule($fieldValue, $ruleValue){
		if(strlen($fieldValue)<$ruleValue){
			return false;
		} else {
			return true;	
		}		
	}
	
	/**
	 * Checks if the string matches the given regexp pattern.
	 */
	public function regexpRule($fieldValue, $ruleValue){
		if(preg_match("$ruleValue", "$fieldValue")){
			return true;
		} else{
			return false;
		}	
	}
	
	/* -------------------------------------------------*/
	/* file upload rules follow */
	
	/**
	 * Checks if the upload is "real".
	 */
	public function upload_nofakeRule($fieldValue, $ruleValue){
		//some checks:
		if($fieldValue == null){
			return false;
		}
		if($fieldValue->getName() == ''){
			return false;
		}
		if($fieldValue->getSize() == null){
			return false;
		}
		if($fieldValue->getError() != UPLOAD_ERR_OK){
			return false;
		}
		
		return is_uploaded_file($fieldValue->getTmpName());
	}
	
	/**
	 * Checks if the upload size does not exceed given.
	 */
	public function upload_maxSizeRule($fieldValue, $ruleValue){
		$fileItem = $fieldValue;
		// first check the error code
		if($fileItem->getError() == UPLOAD_ERR_INI_SIZE || $fileItem->getError() == UPLOAD_ERR_FORM_SIZE){
			return false;
		}
		if($fileItem->getSize() > $ruleValue){
			return false;
		}
		return true;	
	}
	
	/**
	 * Checks if the download file mime type matches against the given regexp pattern.
	 */
	public function upload_typeRule($fieldValue, $ruleValue){

		$out = preg_match("$ruleValue", $fieldValue->getType());
		if($out !== false){
			return true;	
		} else {
			return false;
		}	
	}
}
