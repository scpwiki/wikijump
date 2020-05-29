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

class Captcha extends SmartyScreen {

	private $lx = 100;
	private $ly = 30;
	private $r;
	private $g;
	private $b;

	private $_noise = 5;
	
	private $_minsize=13;
	private $_maxsize=20;

	private $TtfFonts  = array('Vera.ttf','VeraBd.ttf','VeraBI.ttf','VeraIt.ttf','VeraMoBd.ttf','VeraMoBI.ttf','VeraMoIt.ttf', 'VeraMono.ttf', 'VeraSe.ttf', 'VeraSeBd.ttf');

	private $TtfFolder;
	private $TtfFile;

	public function render($runData) {

		$this->TtfFolder = WIKIDOT_ROOT.'/lib/bitstream/';
		
		$runData->setScreenTemplate(null); // this assures no other output will be generated; templating service will NOT run
		$key = $runData->getParameterList()->getParameterValue("key");
		if($key == null || $key == ''){
			$key = 'captchaCode';
		}	
		$ivcode = $runData->sessionGet($key);

		$image =  imagecreatetruecolor($this->lx, $this->ly);

		$this->random_color(224, 255);
		$back =  @imagecolorallocate($image, $this->r, $this->g, $this->b);
		ImageFilledRectangle($image,0,0,$this->lx,$this->ly,$back);

		if($this->_noise > 0){
			// random characters in background with random position, angle, color
			for($i=0; $i < $this->_noise; $i++){
				srand((double)microtime()*1000000);
				$size	= intval(rand((int)($this->_minsize / 2.3), (int)($this->_maxsize / 1.7)));
				srand((double)microtime()*1000000);
				$angle	= intval(rand(0, 360));
				srand((double)microtime()*1000000);
				$x		= intval(rand(0, $this->lx));
				srand((double)microtime()*1000000);
				$y		= intval(rand(0, (int)($this->ly - ($size / 5))));
				$this->random_color(160, 224);
				$color	= imagecolorallocate($image, $this->r, $this->g, $this->b);
				srand((double)microtime()*1000000);
				$text	= chr(intval(rand(45,250)));
				ImageTTFText($image, $size, $angle, $x, $y, $color, $this->changeTtf(), $text);
			}
		}else{
			// generate grid
			for($i=0; $i < $this->lx; $i += (int)($this->_minsize / 1.5)){
				$this->random_color(160, 224);
				$color	= imagecolorallocate($image, $this->r, $this->g, $this->b);
				imageline($image, $i, 0, $i, $this->ly, $color);
			}
			
			for($i=0 ; $i < $this->ly; $i += (int)($this->_minsize / 1.8)){
				$this->random_color(160, 224);
				$color	= imagecolorallocate($image, $this->r, $this->g, $this->b);
				imageline($image, 0, $i, $this->lx, $i, $color);
			}
		}
		
		// generate Text
		for($i=0, $x = intval(rand($this->_minsize,$this->_maxsize)); $i < strlen($ivcode); $i++){
			$text	= strtoupper(substr($ivcode, $i, 1));
			srand((double)microtime()*1000000);
			$angle	= intval(rand(($this->maxrotation * -1), $this->maxrotation));
			srand((double)microtime()*1000000);
			$size	= intval(rand($this->_minsize, $this->_maxsize));
			srand((double)microtime()*1000000);
			$y		= intval(rand((int)($size * 1.5), (int)($this->ly - ($size / 7))));
			$this->random_color(0, 127);
			$color	=  imagecolorallocate($image, $this->r, $this->g, $this->b);
			$this->random_color(0, 127);
			$shadow = imagecolorallocate($image, $this->r + 127, $this->g + 127, $this->b + 127);
			ImageTTFText($image, $size, $angle, $x + (int)($size / 15), $y, $shadow, $this->changeTtf(), $text);
			ImageTTFText($image, $size, $angle, $x, $y - (int)($size / 15), $color, $this->TtfFile, $text);
			$x += (int)($size + ($this->_minsize / 5));
		}

		header("Content-type: image/png");
		header("Expires: Mon, 26 Jul 1997 05:00:00 GMT");
		
		// always modified
		header("Last-Modified: ".gmdate("D, d M Y H:i:s")." GMT");

		// HTTP/1.1
		header("Cache-Control: no-store, no-cache, must-revalidate");
		header("Cache-Control: post-check=0, pre-check=0", false);

		// HTTP/1.0
		header("Pragma: no-cache");
		
		imagepng($image);
		imagedestroy($image);

	}

	public function build($runData){}
	
	private function random_color($min,$max){
		srand((double)microtime() * 1000000);
		$this->r = intval(rand($min,$max));
		srand((double)microtime() * 1000000);
		$this->g = intval(rand($min,$max));
		srand((double)microtime() * 1000000);
		$this->b = intval(rand($min,$max));
	}
	
	private function changeTtf(){
		if(is_array($this->TtfFonts)){
			srand((float)microtime() * 10000000);
			$key = array_rand($this->TtfFonts);
			$this->TtfFile = $this->TtfFolder.$this->TtfFonts[$key];
			
			if(!file_exists($this->TtfFile)){
				echo "file ".	$this->TtfFile . ' does not exist';
			}
			
		}else{
			$this->TtfFile = $this->TtfFolder.$this->TtfFonts;
		}
		return $this->TtfFile;
	}

}
