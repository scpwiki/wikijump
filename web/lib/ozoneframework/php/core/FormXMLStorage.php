<?php



/**
 * A gelper class for XML data storage (definition) of the forms. The only
 * reason for this class is to make the Form object lighter and to store only
 * necessary information and NOT parsed xml with the form definition.
 */
class FormXMLStorage {
	public static $storage = array();

	public static function initForm($formName){
		$fileName = PathManager::formSpecFile($formName);
		$xml = simplexml_load_file($fileName);

		self::$storage["$formName"]=array();
		$formxml = $xml->form[0];
		self::$storage["$formName"]['xml'] = $formxml;

		// refactor just a bit for an easy access to fields
		$fields = array();
		$fieldNames = array();
		foreach ($formxml as $field){
			$tname = $field['name'];
			$fieldNames[] = $tname;
			$fields["$tname"] = $field;
		}
		self::$storage["$formName"]['fields'] = $fields;
		self::$storage["$formName"]['fieldNames'] = $fieldNames;
	}

	public static function getFormXML($formName){
		if(self::$storage["$formName"] == null){
			self::initForm($formName);
		}
		return self::$storage["$formName"]['xml'];
	}

	public static function getFormFields($formName){
		if(self::$storage["$formName"] == null){
			self::initForm($formName);
		}
		return self::$storage["$formName"]['fields'];
	}

	public static function getFormFieldNames($formName){
		if(self::$storage["$formName"] == null){
			self::initForm($formName);
		}
		return self::$storage["$formName"]['fieldNames'];
	}

}
