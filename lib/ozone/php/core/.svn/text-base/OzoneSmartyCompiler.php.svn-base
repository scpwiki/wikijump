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
require_once (SMARTY_DIR . '/Smarty_Compiler.class.php');

/**
 * Modified compiler for the Smarty.
 *
 */
class OzoneSmartyCompiler extends Smarty_Compiler {

	/**
	 * Allow static method calls.
	 */
	function _parse_attrs($tag_args) {
		$attrs = parent :: _parse_attrs($tag_args);

		foreach ($attrs as $key => $value) {
			// perhaps this was intended as a static callback?
			if (preg_match('#^["\']([a-zA-Z_]\w*::[a-zA-Z_]\w*)\((.*)?\)["\']$#', $value, $matches)) {
				$arguments = '()';
				if (isset ($matches[2])) {
					// strip '".' and '."' from beginning and end
					$arguments = substr($matches[2], 2, -2);

					// remove '.",".' from between parameters
					$arguments = explode('.",".', $arguments);

					// combine arguments into string
					$arguments = '(' . implode(',', $arguments) . ')';
				}

				$attrs[$key] = $matches[1] . $arguments;
			}
		}
		return $attrs;
	}

}
