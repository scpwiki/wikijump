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

class CryptUtils {
	private static $keyFile = 'files/key.pem';
	private static $publicFile = 'files/public.pem';
	
	public static function generateSeed($length=10){
		$vals = '0123456789abcdefghijklmnopqrstuvwxyz';
		$charMin = 48;
		$charMax = 90;
		$out = '';
		for($i = 0; $i<$length; $i++){
			$out .= $vals{rand(0, 35)};	
		}
		return $out;
	}
	
	public static function rsaGenerateModulus(){
		$keyFile = WIKIDOT_ROOT.'/'.self::$keyFile;
		$keyFile = escapeshellarg($keyFile);
		$cmd = 'openssl rsa -in '.$keyFile.' -noout -modulus';
		$modulus =  exec($cmd);
		$modulus = trim(str_replace("Modulus=", '', $modulus));
		return $modulus;
	}

	public static function rsaDecrypt($text){
		$keyFile = WIKIDOT_ROOT.'/'.self::$keyFile;
		$keyFile = escapeshellarg($keyFile);
		$cmd = 'openssl base64 -d | openssl rsautl -inkey '.$keyFile.' -decrypt';
		$descriptorspec = array(
   			0 => array("pipe", "r"),  // stdin is a pipe that the child will read from
   			1 => array("pipe", "w"),  // stdout is a pipe that the child will write to
   			2 => array("pipe", "w") // stderr is a file to write to
		);
		$process = proc_open($cmd, $descriptorspec, $pipes);
		if(!is_resource($process)){
			throw new Exception(_("RSA decryption failed."));
		}
		fwrite($pipes[0], $text);
   		fclose($pipes[0]);

   		$error = stream_get_contents($pipes[2]);
   		if($error !== null && $error !== ''){
   			throw new Exception(_("RSA decryption failed").": ".$error);	
   		}
			
		$result =  stream_get_contents($pipes[1]);
		fclose($pipes[1]);
		return $result;
	}
	
	public static function rsaEncrypt($text){
		$keyFile = WIKIDOT_ROOT.'/'.self::$publicFile;
		$keyFile = escapeshellarg($keyFile);
		$cmd = 'openssl rsautl -pubin -inkey '.$keyFile.' -encrypt | openssl base64 -e';
		$descriptorspec = array(
   			0 => array("pipe", "r"),  // stdin is a pipe that the child will read from
   			1 => array("pipe", "w"),  // stdout is a pipe that the child will write to
   			2 => array("pipe", "w") // stderr is a file to write to
		);
		$process = proc_open($cmd, $descriptorspec, $pipes);
		if(!is_resource($process)){
			throw new Exception(_("RSA encryption failed."));
		}
		fwrite($pipes[0], $text);
   		fclose($pipes[0]);

   		$error = stream_get_contents($pipes[2]);
   		if($error !== null && $error !== ''){
   			throw new Exception(_("RSA decryption failed").": ".$error);	
   		}
			
		$result =  stream_get_contents($pipes[1]);
		fclose($pipes[1]);
		return $result;
	}
	
	public static function modulus(){
	    $m = file_get_contents(WIKIDOT_ROOT.'/files/modulus.pem');
	    return trim($m);
	}

}
