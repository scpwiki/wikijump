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

class LatexRenderer {
	private $latexPath = "/usr/bin/latex";
	private $dvipsPath = "/usr/bin/dvips";
	private $convertPath = "/usr/bin/convert";
	private $tmpDir = "/tmp/latex-renderer";
	private $outputDir = "/usr/home/barik/public_html/gehennom/lj/cache";

	private $density = 130;

	public function wrap($thunk) {
		return<<<EOS
\documentclass[10pt]{article}
% add additional packages here
\usepackage{amsmath}
\usepackage{amsfonts}
\usepackage{amssymb}
\usepackage{pst-plot}
\usepackage{color}
\pagestyle{empty}
\begin{document}
$thunk
\end{document}
EOS;
	}

	function render($thunk, $hash) {
		$thunk = $this->wrap($thunk);
		$current_dir = getcwd();
		chdir($this->tmpDir);
		// create temporary LaTeX file
		$fp = fopen($this->tmpDir."/$hash.tex", "w+");
		fputs($fp, $thunk);
		fclose($fp);
		// run LaTeX to create temporary DVI file
		$command = $this->latexPath." --interaction=nonstopmode ".$hash.".tex";
		exec($command);
		if(!file_exists($hash.".dvi")){
			return false;
		}
		// run dvips to create temporary PS file
		$command = $this->dvipsPath." -E $hash".".dvi -o "."$hash.eps";
		exec($command);
		// run PS file through ImageMagick to
		// create PNG file
		$command = $this->convertPath." -verbose -density ".$this->density." ".$hash.".eps ".$hash.".png 2>&1";

		exec($command, $out);

		// copy the file to the cache directory
		if(!file_exists($hash.".png")){
			return false;
		}
		copy("$hash.png", $this->outputDir."/$hash.png");
		chdir($current_dir);
		$this->cleanup($hash);
	}
	
	function cleanup($hash) {

		unlink($this->tmpDir."/$hash.tex");
		unlink($this->tmpDir."/$hash.aux");
		unlink($this->tmpDir."/$hash.log");
		unlink($this->tmpDir."/$hash.dvi");
		unlink($this->tmpDir."/$hash.eps");
		unlink($this->tmpDir."/$hash.png");

	}

	public function setTmpDir($tmpdir){
		$this->tmpDir = $tmpdir;	
	}
	
	public function setOutputDir($outdir){
		$this->outputDir = $outdir;	
	}
	
	public function setDensity($val){
		$this->density = $val;
	}

}
