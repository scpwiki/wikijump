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
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

/**
 * A set of methods handling diff operations.
 *
 */
class Wikidot_Util_Diff {
	
	/**
	 * Implementation of unified diff of two strings
	 * 
	 * @param $fromString the first string to create diff from
	 * @param $toString the second string to create diff from
	 * @param $contextLines the number of lines of context
	 * @param $minimal whether to find the minimal diff or just any good
	 * @return string the unified diff
	 */
	static public function unifiedDiff($fromString, $toString, $contextLines = 3, $minimal = false) {
#		Implementation of unified diff of two strings

#		using php libxdiff:
#		
#		if(!extension_loaded('xdiff')){
#			dl( 'xdiff.so' );
#		}
#		return xdiff_string_diff($fromString, $toString, $contextLines, $minimal);

#		or the diff command:
#
		$file_from = tempnam(WIKIDOT_ROOT . '/tmp', 'diff-');
		$file_to = tempnam(WIKIDOT_ROOT . '/tmp', 'diff-');
		file_put_contents($file_from, $fromString);
		file_put_contents($file_to, $toString);
		
		$from_arg = escapeshellarg($file_from);
		$to_arg = escapeshellarg($file_to);
		$minimal_arg = $minimal ? "-d" : "";
		$context_arg = (int) $contextLines;
		$cmd = "diff $minimal_arg -U $context_arg $from_arg $to_arg";
		
		$result_lines = array();
		exec($cmd, $result_lines);
        array_shift($result_lines);
        array_shift($result_lines);
		
		unlink($file_from);
		unlink($file_to);
		
		return implode("\n", $result_lines);
	}
	
	/**
	 * Generates a difference between two strings.
	 *
	 * @param string $fromString
	 * @param string $toString
	 * @return string
	 */
	static public function generateStringDiff($fromString, $toString, $contextLines = 1, $minimal = true){
		// fix "no new line at the end" problem.
		$fromString = Wikidot_Util_String::addTrailingNewline($fromString);
		$toString = Wikidot_Util_String::addTrailingNewline($toString);
		
		return self::unifiedDiff($fromString, $toString, $contextLines, $minimal);
	}
	
	/**
	 * Patches (or reverse-patches) string with a diff.
	 *
	 * @param string $string
	 * @param string $patch
	 * @param bool $reverse
	 * @return string
	 */
	static public function patchString($string, $patch, $reverse = false){
		// fix "no new line at the end" problem.
		$string = Wikidot_Util_String::addTrailingNewline($string);
		$patch = Wikidot_Util_String::addTrailingNewline($patch);
		
		if($reverse == false){
			$flags = XDIFF_PATCH_NORMAL;
		}else{
			$flags = XDIFF_PATCH_REVERSE;
		}
		$errors = array();
		$r = xdiff_string_patch($string, $patch, $flags, &$errors);
		if(count($errors) > 0){
			throw new Wikidot_Exception("Error while applying the patch.");
		}
		return $r;
	}
	
	/**
	 * Generates a nice inline diff.
	 * The config options are
	 *  - noChange = true - does not create 'change' blocks which uses word-level diffs
	 *  - asArray = false - outputs the an array of lines insted of text 
	 * 
	 * @param string $fromString
	 * @param string $toString
	 * @param array $config
	 * @return string|array
	 */
	static public function generateInlineStringDiff($fromString, $toString, $config = array()){
		
		$useChange = (isset($config['noChange'])&&$config['noChange']==true)?false:true;
		$outputAsArray = (isset($config['asArray'])&&$config['asArray']==true)?true:false;
		
		$xi = $yi = 1;
		$block = false;
		$context = array();

		$nlead = 10000;
		$ntrail = 10000;

		$output = array();

		// make a diff with the FULL output included too.
		$diff = Wikidot_Util_Diff::generateStringDiff($fromString, $toString, count(explode("\n", $toString)));
		
		$diffs2 = explode("\n", $diff);
		$diffs = array();
		for($i = 0; $i < count($diffs2); $i++){
			$d = $diffs2[$i];
			if(strlen($d) == 0){
				continue;
			}
			$type = null;
			switch($d{0}){
				case ' ':
					$type = 'copy';
					break;
				case '-':
					$type = 'delete';
					break;
				case '+':
					$type = 'add';
					break;
			}
			if($type){
				// handle a special situation if the previous line was 'delete' and this
				// one is 'add' - change this to 'change'.
				$c = count($diffs);
				if($useChange && $c>0 && $type == 'add' && $diffs[$c - 1]['type'] == 'delete'){
					$diffs[$c - 1]['type'] = 'change';
					$diffs[$c - 1]['toline'] = substr($d, 1);
				}else{
					$diffs[] = array('type' => $type, 'line' => substr($d, 1));
				}
			}

		}
		
		// generate output
		$output = array();
		$currentType = 'copy';
		$countDiffs = count($diffs);
		for($i = 0; $i < $countDiffs; $i++){
			$row = '';
			$d = $diffs[$i];
			$type = $d['type'];
			if($type != $currentType){
				switch($type){
					case 'add':
						$row .= '<ins>';
						break;
					case 'delete':
						$row .= '<del>';
						break;
				}
				$currentType = $type;
			}
			
			if($type == 'change'){
				//special treatment
				$line = preg_replace(';(?<!\s[^\s]{1}|\s[^\s]{2})(\s+);', "\\1\n", $d['line']);
				$toline = preg_replace(';(?<!\s[^\s]{1}|\s[^\s]{2})(\s+);', "\\1\n",$d['toline']);
				// process this too
				$linediff = self::generateInlineStringDiff($line, $toline, array('asArray' => true, 'noChange' => true));
				$row .= implode('', $linediff);
			}else{
				$row .= htmlspecialchars($d['line']);
			}
			
			if($i<$countDiffs - 1){
				$nextType = $diffs[$i+1]['type'];
			}else{
				$nextType = null;
			}
			if($type != $nextType){
				// close the type
				switch($type){
					case 'add':
						$row .= '</ins>';
						break;
					case 'delete':
						$row .= '</del>';
						break;
				}
			}
			$output[] = $row;
			
		}

		if($outputAsArray){
			return $output;
		}else{
			return implode("\n", $output);
		}

	}
	
}
