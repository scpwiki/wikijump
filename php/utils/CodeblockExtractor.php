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
 * @category Wikidot
 * @package Wikidot
 * @version $Id: CodeblockExtractor.php,v 1.2 2008/08/05 21:00:26 quake Exp $
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

class CodeblockExtractor {

	protected $mimeType = null;
	protected $contents = "";
	protected $treatAsTemplate = false;
	protected $templateVariables = array();
	
	protected $mimeMap = array(
		"css"	=> "text/css",
		"html"	=> "text/html",
	);
	
	public function __construct($site, $pageName, $codeblockNo = 1, $templateVars = null){
		try {
			$codeblockNo = (int) $codeblockNo;
			if ($codeblockNo < 1) {
				$codeblockNo = 1;
			}
			
			$page = DB_PagePeer::instance()->selectByName($site->getSiteId(), $pageName);
			
			if($page == null){
				throw new ProcessException("No such page");
			}
			// page exists!!! wooo!!!
			
			$source = $page->getSource();
			/* Get code block. */
			
			$regex = ';^\[\[code(\s[^\]]*)?\]\]((?:(?R)|.)*?)\[\[/code\]\](\s|$);msi';
			
			$allMatches = array();
			preg_match_all($regex, $source, $allMatches);
			
			if(count($allMatches[2]) < $codeblockNo) {
				throw new ProcessException('No valid codeblock found.');
			}
			
			$code = $allMatches[2][$codeblockNo - 1];
			if ($allMatches[1][$codeblockNo - 1]) {
				$params = $allMatches[1][$codeblockNo - 1];
				$m = array();
				$type = null;
				if (preg_match(':type="([^"]+)":', $params, $m)) {
					$type = strtolower($m[1]);
				}
				if (array_key_exists($type, $this->mimeMap)) {
					$this->mimeType = $this->mimeMap[$type];
				}
			}
			
			$code = trim($code) . "\n";
			
			if (is_array($templateVars)) {
				$this->contents = $this->renderFromTemplate($code, $templateVars);
			} else {
				$this->contents = $code;
			}
			
		} catch(Exception $e) {
			$this->contents = $e->getMessage();
		}
	}
	
	public function getContents() {
		return $this->contents;
	}
	
	public function getMimeType() {
		if ($this->mimeType) {
			return $this->mimeType;
		}
		return "text/plain";
	}
	
	public function renderFromTemplate($template, $extValues) {
		$template = "\n$template\n";
		$template_parts = explode("\n---\n", $template);
		
		// form definition is the YAML document before the first "---"
		$form_def = array_shift($template_parts);
		
		// Wikidot (DTL) template is the rest
		$template = trim(implode("\n---\n", $template_parts));
		
		$form = Wikidot_Form::fromYaml($form_def);
		$context = $form->computeValues($extValues);
		
		// render the template
		$w_template = new Wikidot_Template($template);
		return $w_template->render($context);
	}
}
