<?php

if(empty($_REQUEST['debug'])){
	error_reporting(1);
	header("Content-Type: image/png"); 
}

require_once("src/image-lib.php");

$defaults = array(
	"src" => "images/square.png",
	// skew: one of left, right otherwise doesn't skew
	"skew" => "none",
	"reflect" => "1",
	"angle" => 10,
	"spread" => 0.75,
	"decay" => 25,
	"spacing" => 0,
	"refdir" => 1,
	"thumbsize" => 100,
	"greyscale" => 0
);

forEach($defaults as $key => $pair){
	$$key = (empty($_REQUEST[$key]) ? $pair : $_REQUEST[$key]);	
}

$cachefile = "cache/" . md5(
		$src . 
		$skew .
		$reflect . 
		$angle . 
		$spread . 
		$decay . 
		$spacing . 
		$refdir . 
		$thumbsize .
		$greyscale 
	) . ".png";

if(file_exists($cachefile)){

	$mod = filemtime($cachefile);
	$cached = !empty($_SERVER['HTTP_IF_MODIFIED_SINCE']);
	$resp = $cached ? 304 : 200;
	
	header("Last-Modified: " . gmdate('D, d M Y H:i:s', $mod) . "GMT", true, $resp);
	
	if(!$cached){
		header("Content-Length: " . filesize($cachefile));
		if($fp = fopen($cachefile, "r")){
			while(!feof($fp)){
				print fgets($fp, 2048);
			}
		}
	}
	exit;
	
}else{
	
	$ext = strtolower(substr($src, -3, 3));
	switch($ext){
		case "png" : $image = imagecreatefrompng($src); break;
		case "jpg" : $image = imagecreatefromjpeg($src); break;
	}

	if(!empty($image)){
		
		$im = imageReflect(imageSquareThumb(($greyscale==1 ? imageGreyscale($image) : $image), $thumbsize), $refdir, $spread, $decay, $spacing);

		$dir = false;

		switch($skew){
			case "left" : $dir = 1; break;
			case "right" : $dir = 2; break;
		}
	
		if($dir){
			$display = imageSkew($im, $angle, $dir - 1);
		}else{
			$display = $im;
		}

		imagepng($display, $cachefile);
		imagepng($display);
	}
}

?>