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
 * @package Ozone_Form
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */
 
/**
 * Form object.
 *
 */
class Form {
	
//	private $formxml; //definition of the form!
	
	private $name;
	private $formKey="_0"; //default formKey
	
	private $fields;
	
	private $fieldValues = array();
	
	private $errorMessages = array();
	private $isValidAll = true;
	private $isValidArray = array();
	
	private $validated = false;
	
	private $validatorName = null;
	private $extraValidatorName = null;
	
	private $retrieved = false;
	
	/**
	 * If the form is resubmitted (by clicking "reload" button).
	 */
	private $resubmitted = false;

	public function __construct($formName, $formKey = "_0"){
		$this->name = $formName;
		$this->formKey = $formKey;
	}
	
	public function getName(){
		return $this->name;	
	}
	
	public function setName($name){
		$this->name = $name;
	}
	
	public function getFormKey(){
		return $this->formKey;	
	}
	
	public function setFormKey($formKey){
		$this->formKey = $formKey;
	}

	public function getFieldType($name){
		$fields = FormXMLStorage::getFormFields($this->name);
		return $fields["$name"]->rendering[0]['type'];
	}
	
	public function getFieldValue($name){
		$tvalue = $this->fieldValues["$name"];
		$fieldType = $this->getFieldType($name);
		if($fieldType == 'text' || $fieldType == 'password' || $fieldType == 'textarea'|| $fieldType=='select'|| $fieldType=='hidden'){
			if($tvalue !== null){
				return $tvalue;
			} else{
				$fields = FormXMLStorage::getFormFields($this->name);
				$tvalue = $fields["$name"]['defaultValue'];
				return $tvalue;	
			}
		} 
		if($fieldType=='checkbox'){
			if($tvalue !== null){
				return $tvalue;
			} else{
				$fields = FormXMLStorage::getFormFields($this->name);
				$tvalue = $fields["$name"]['defaultValue'];
				if($tvalue == 'true' || $tvalue == 'on' || $tvalue == 'yes'){
					return true;
				}else{
					return false;
				}	
			}	
		}
		
		if($fieldType=='file'){
			$fu = new FileUpload();
			return $fu->getFileItem($this->getFieldLabel($name));	
		}
		
	}
	
	public function setFieldValue($fieldName, $value){
		$this->fieldValues["$fieldName"] = trim($value);	
	}

	public function getFieldTitle($name){
		$fields = FormXMLStorage::getFormFields($this->name);
		$text = xml_localized_text($fields["$name"]->title);
		return trim($text);	
	}
	
	public function getFieldSubTitle($name){
		$fields = FormXMLStorage::getFormFields($this->name);
		$text = xml_localized_text($fields["$name"]->subtitle);
		return trim($text);	
	}
	
	public function getFieldLabel($name){
		return $this->name . $this->formKey . $name;	
	}
	
	public function getFieldMaxLength($name){
		$fields = FormXMLStorage::getFormFields($this->name);
		return $ields[$name]->rendering[0]['maxlength'];	
	}
	
	/**
	 * Returns additional field attribute as defined in the <extra attribute="value".../> tag 
	 * for a given field.
	 * @param string $fieldName
	 * @param string $attributeName
	 * @return string attribute value
	 */
	public function getExtraAttribute($fieldName, $attributeName){
		$fields = FormXMLStorage::getFormFields($this->name);
		return $fields["$fieldName"]->extra[0]["$attributeName"];
	}
	
	/** 
	 * Returns true if the form has been retrieved via user submission. If
	 * the form is new - returns false.
	 */
	public function isRetrieved(){
		return $this->retrieved;	
	}

	public function setRetrieved($retrieved){
		$this->retrieved = $retrieved;	
	}
	
	public function isNew(){
		return ! $this->retrieved;	
	}

	public function validate($fieldName = null){
		$this->validated = true;
		$fieldNames = FormXMLStorage::getFormFieldNames($this->name);
		$fields = FormXMLStorage::getFormFields($this->name);
		if($fieldName == null){
			$this->errorMessages = array();
			// validate all the fields
			$this->isValidAll = true;
			foreach ( $fieldNames as $fname){
				$this->validate($fname);
				if($this->isValidArray["$fname"] == false){
					$this->isValidAll = false; // one false is enough to spoil the whole form!	
				}
			}
				
		} else {
			//ok, validate the field $fieldName
			$this->isValidArray["$fieldName"] = true;	
			// get rule-chain
			if($fields["$fieldName"]->validation[0] == null){
				$this->isValidArray["$fieldName"] = true;
				return true; // no validation required for this field	
			}
			
			$chain = $fields["$fieldName"]->validation[0];

			if($this->validatorName == null) {
				$this -> validatorName = "BaseFormValidator";	
			}
				
			foreach($chain as $rule){
				$this->isValidArray["$fieldName"]=true;
				$ruleName = $rule['name'];
				$ruleValue = $rule['value'];
				// now perform the validation
				
				$validator = new $this->validatorName();
				$ruleMethod = $ruleName.'Rule';
				$validationResult_sub = $validator->$ruleMethod($this->getFieldValue($fieldName), $ruleValue);
				if($validationResult_sub == false){
					// and save the validation result!!!
					$this->isValidArray["$fieldName"] = false;	
					// and set the error message!
					$this->errorMessages["$fieldName"] = "".xml_localized_text($rule->message);
					return;
				}
			}
			
		}
	}
	
	public  function isValid($fieldName = null){
		if($this->validated == false){
			return true;	
		}
		
		if($fieldName == null){
			$this->updateValidAll();
			return $this->isValidAll;	
		} else {
			return $this->isValidArray["$fieldName"];	
		}
			
	} 
	
	public function setValid($fieldName, $value){
		$this->isValidArray["$fieldName"] = $value;
	}
	
	public function getErrorMessage($fieldName){
		return trim($this->errorMessages[$fieldName]);	
	}
	
	public function getErrorMessages(){
		return $this->errorMessages;	
	}
	
	public function setErrorMessage($fieldName, $message){
		$this->errorMessages[$fieldName] = $message;	
	}
	
	public function declarations(){
		$out="";
		$out.='<input type="hidden" name="formname" value="'.$this->name  .'"/>';	
		$out.='<input type="hidden" name="formkey" value="'.$this->formKey  .'"/>';	
		$out.='<input type="hidden" name="use_formtool" value="yes"/>';
		
		// put an UNIQUE key to allow resubmission detection
		$key = UniqueStrings::timeBased();
		$out.='<input type="hidden" name="form_submission_key" value="'.$key.'"/>';
		
		return $out;
	}
	
	public function renderingString($name){
		
		$fields = FormXMLStorage::getFormFields($this->name);
		$attributes = $fields[$name]->rendering[0]->attributes();
		$out = "";
		$fieldType = $this->getFieldType($name);
		if($fieldType == 'text' || $fieldType == 'password'|| $fieldType == 'checkbox' || $fieldType=='hidden'){
			if($attributes !== null){
				foreach($attributes as $key => $value){
					$out.=' '.$key.'="'.$value.'" ';
					
				}
			}
		}
		
		if($fieldType=="select" || $fieldType=="textarea"){
			if($attributes !== null){
				foreach($attributes as $key => $value){
					if($key!=type){
						$out.=' '.$key.'="'.$value.'" ';
					}
					
				}
			}
		}

		if($fieldType == 'file'){
			if($attributes !== null){
				foreach($attributes as $key => $value){
					$out.=' '.$key.'="'.$value.'" ';
					
				}
			}	
		}
		
		return $out;
	}
	
	public function isValidated(){
		return $this->validated;
	}	
	
	public function setValidated($value){
		$this->validated = $value;	
	}
	
	public function populateFromParameterArray($parameterArray){
		$fieldNames = FormXMLStorage::getFormFieldNames($this->name);
		foreach ($fieldNames as  $key){
			$paramKey = $this->name.$this->formKey.$key;
			$fieldType = $this->getFieldType($key);
			
			if($fieldType == 'text' || $fieldType == 'password' || $fieldType=='textarea' || $fieldType=='select' || $fieldType=="hidden"){
				$tmp1 = $parameterArray["$paramKey"];
				if($tmp1 != null){
					$this->fieldValues["$key"] = trim($tmp1);
				} else {
					$this->fieldValues["$key"] = null;
				}
			}
			
			if($fieldType == 'checkbox'){
				if(isset($parameterArray["$paramKey"])){
					$this->fieldValues["$key"] = true;
				} else {
					$this->fieldValues["$key"] = false;	
				}
			}
			
		}
	}
	
	private function updateValidAll(){
		$validAll = true;
		$fieldNames = FormXMLStorage::getFormFieldNames($this->name);
		foreach ( $fieldNames as $fname){
			if($this->isValidArray["$fname"] == false){
				$validAll = false;
			}
		}
		$this->isValidAll = $validAll;
	}
	
	public function getHelpText($fieldName){
		$fields = FormXMLStorage::getFormFields($this->name);
		$text = xml_localized_text($fields["$fieldName"]->help);
		return trim(''.$text);	
	}
	
	public function getAllHelpTexts(){
		$fields = FormXMLStorage::getFormFields($this->name);
		$fieldNames = FormXMLStorage::getFormFieldNames($this->name);
		$helps = array();
		foreach ($fieldNames as $fieldName){
			$helps["$fieldName"] = 	$trim(''.$fields["$fieldName"]->help[0]);
		}
		return $helps;	
	}
	
	public function getFieldNames(){
		return  FormXMLStorage::getFormFieldNames($this->name);	
	}
	
	/**
	 * Return associated list name for a select element.
	 */
	public function getSelectValueListName($fieldName){
		$fields = FormXMLStorage::getFormFields($this->name);
		return $fields["$fieldName"]['valueList'];
		
	}
	
	/**
	 * Return associated table name for a select element.
	 */
	public function getSelectValueTableName($fieldName){
		$fields = FormXMLStorage::getFormFields($this->name);
		return $fields["$fieldName"]['valueTable'];
		
	}
	
	/**
	 * When uploading files - this determines the max size (in bytes) allowed to 
	 * download. Specified in the -form.xml file in the validation rule chain.
	 */
	public function getUploadMaxSize($name){
		$fields = FormXMLStorage::getFormFields($this->name);
		$rules = $fields["$name"]->validation[0]->rule;
		$mrule = findNodeWithAttribute($rules, 'name', 'upload_maxsize');
		if ($mrule == null) {return null;}
		return $mrule['value'];
	}
	
	public function isUpload(){
		$formXML = FormXMLStorage::getFormXML($this->name);
		$ustring = $formXML['upload'];
		if($ustring == 'true'){
			return true;
		}else{
			return false;
		}
	}
	
	public function setResubmitted($value){
		$this->resubmitted = $value;	
	}
	
	public function isResubmitted(){
		return $this->resubmitted;
	}	
}
