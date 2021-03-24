<?php

namespace Ozone\Framework;



/**
 * Form upload tool.
 *
 */

use Ozone\Framework\Database\Criteria;
use Wikidot\DB\FormSubmissionKey;
use Wikidot\DB\FormSubmissionKeyPeer;

class FormTool {

	// should become true after a form is substracted from the http request

	private $formStorage = array();

	public function hasForms(){}

	public function getForm($name, $formKey=null){
		if($formKey == null) {$formKey = "_0";} // set default formKey
		if($this->formStorage["$name"] == null || $this->formStorage["$name"]["$formKey"] == null){
			return $this->newForm($name, $formKey);
		} else {
			$form = $this->formStorage["$name"]["$formKey"];
			$form->setRetrieved(true);
			return $form;
		}
	}

	public function delForm($formName=null, $formKey=null) {
		if($formName != null && $formKey != null){
			unset($this->formStorage["$formName"]["$formKey"]);
		}
		if($formName != null && $formKey == null){
			unset($this->formStorage["$formName"]);
		}
		if($formName == null){
			//clear all
			$this->formStorage = array();
		}
	}

	public function newForm($formName, $formKey="_0") {

		$form = new Form($formName, $formKey);
		$form->setFormKey($formKey);

		$form->setRetrieved(false);
		return $form;

	}

	public function processHttpRequest($runData){
		$parameters = $runData->getParameterList()->asArray();
		if($parameters['use_formtool'] == yes){
			$formName = $parameters['formname'];
			$formKey = $parameters['formkey'];
			$form = $this->getForm($formName, $formKey);
			//populate the form
			$form->populateFromParameterArray($parameters);
			// check if resubmitted
			$key = $parameters['form_submission_key'];
			$c = new Criteria();
			$c->add("key_id", $key);
			$entry = FormSubmissionKeyPeer::instance()->selectOne($c);
			if($entry == null){
				$form->setResubmitted(false);
				// insert key into database
				$entry = new FormSubmissionKey();
				$entry->setKeyId($key);
				$entry->setDateSubmitted(new ODate());
				$entry->save();
			} else {
				$form->setResubmitted(true);
			}
			//save form to the storage
			$this->formStorage["$formName"] = array();
			$this->formStorage["$formName"]["$formKey"]=$form;
		}

	}

	/** Returns the number of stored forms */
	public function formsNumber(){
		return count($this->formStorage);
	}

	/** Resets the state of all stored forms to non-validated. */
	public function resetFormsToNonvalidated(){
		foreach ($this->formStorage as $forms2){
			foreach ($forms2 as $form){
				$form->setValidated(false);
			}
		}
	}
}
