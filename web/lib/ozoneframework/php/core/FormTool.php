<?php

namespace Ozone\Framework;



/**
 * Form upload tool.
 *
 */

class FormTool {

	// should become true after a form is substracted from the http request

	private $formStorage = array();

	public function getForm($name, $formKey=null){
		if($formKey == null) {$formKey = "_0";} // set default formKey
		if($this->formStorage["$name"] == null || $this->formStorage["$name"]["$formKey"] == null) {
			return $this->newForm($name, $formKey);
		} else {
			$form = $this->formStorage["$name"]["$formKey"];
			$form->setRetrieved(true);
			return $form;
		}
	}

	public function newForm($formName, $formKey="_0") {
		$form = new Form($formName, $formKey);
		$form->setFormKey($formKey);
		$form->setRetrieved(false);
		return $form;
	}

	public function processHttpRequest($runData) {
		$parameters = $runData->getParameterList()->asArray();
		if($parameters['use_formtool'] == yes){
			$formName = $parameters['formname'];
			$formKey = $parameters['formkey'];
			$form = $this->getForm($formName, $formKey);
			//populate the form
			$form->populateFromParameterArray($parameters);
			//save form to the storage
			$this->formStorage["$formName"] = array();
			$this->formStorage["$formName"]["$formKey"] = $form;
		}
	}
}
