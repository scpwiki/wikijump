<?php

require_once(WIKIDOT_ROOT . "/lib/h2o/h2o.php");
require_once(WIKIDOT_ROOT . "/lib/h2o/h2o/parser.php");

class Wikidot_Template {
	/**
	 * H2o instance
	 * @var H2o
	 */
	protected $template;
	
	/**
	 * constructs the template from given source
	 * 
	 * @param $templateString string -- the template source
	 */
	public function __construct($templateString = "") {
		$this->template = H2o::parseString($templateString, array("autoescape" => false));
	}
	
	/**
	 * renders the template in the given context
	 * 
	 * @param $context array or object of context
	 * @return string the rendered template
	 */
	public function render($context = array()) {
		return $this->template->render($context);
	}
}
