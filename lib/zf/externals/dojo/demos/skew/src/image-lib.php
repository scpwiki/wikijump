<?php

	/* my New BSD Image library ... */

	function imageGreyscale($im){

		   // get image dimensions
		   $w  = imagesx($im);
		   $h = imagesy($im);

		   $canvas = imagecreatetruecolor($w, $h);

		   for ($y = 0; $y < $h; $y++) {
			   for ($x = 0; $x < $w; $x++) {

				   $rgba = imagecolorat($im, $x, $y);
				   $r = ($rgba >> 16) & 0xFF;
				   $g = ($rgba >> 8)  & 0xFF;
				   $b = $rgba & 0xFF;

				   $gray = round(.299 * $r + .587 * $g + .114 * $b);

				   // shift gray level to the left
				   $grayR = $gray << 16;  // R: red
				   $grayG = $gray << 8;	   // G: green
				   $grayB = $gray;		  // B: blue
				   $grayColor = $grayR | $grayG | $grayB;

				   // set the pixel color
				   imagesetpixel($canvas, $x, $y, $grayColor);
				   imagecolorallocate($canvas, $gray, $gray, $gray);
			   }
		   }
	
			return $canvas;
	
	}

	function imageSquareThumb($im, $size){
		// Creates a size x size thumbnail scaled from the image passed and returns another image
		$ow = imagesx($im);
		$oh = imagesy($im);
	
		$thumb = imagecreatetruecolor($size,$size);
		if($ow > $oh){
		   $off_w = ($ow-$oh)/2;
		   $off_h = 0;
		   $ow = $oh;
		}elseif($oh > $ow){
		   $off_w = 0;
		   $off_h = ($oh-$ow)/2;
		   $oh = $ow;
		}else{
		   $off_w = 0;
		   $off_h = 0;
		}
		imagecopyresampled($thumb, $im, 0, 0, $off_w, $off_h, $size, $size, $ow, $oh);
		return $thumb;
	
	}

	function imageReflect($im, $dir = 1, $spread = 0.55, $decay = 15, $spacing = 0){
		// Reflect an image pointer, returning a slightly larger image
		//
		// $dir: 1 == "vertical" reflection, 2 == "horizontal"
		// $spread: % to reflect. defaults to 0.55, or 55% original image
		// $decay: A value to adjust initial opacity and decaying visibility. lower == more visible
	
		$w = imagesx($im);
		$h = imagesy($im);

		$vert = $dir & 1;
		$hori = $dir & 2;

		// calculate the size of our attachment
		$nw = $vert ? $w : $w * $spread;
		$nh = $hori ? $h : $h * $spread;
	
		// add our reflection size to the height or width
		$fw = $w + ($vert ? 0 : $nw);
		$fh = $h + ($hori ? 0 : $nh);

		$reflect = imagecreatetruecolor($fw, $fh);
		imagealphablending($reflect, false);
		$trans = imagecolorallocatealpha($reflect, 0, 0, 0, 127);
		imageFill($reflect, 0, 0, $trans);

		// put the orig on canvas at 0x0
		imagecopy($reflect, $im, 0, 0, 0, 0, $w, $h);
	
		if($vert){
			// vertical
			for($y = 0; $y < $nh; $y++){
			
				$opacity = $decay + ceil(127 * (($y + $decay) / $h));
				if($opacity > 127){ $opacity = 127; }
			
				for($x = 0; $x < $nw; $x++){

					$rgba = imagecolorat($im, $x, $h - $y - 1);
				
					// break out to function:
					$r = ($rgba & 0xFF0000) >> 16;
					$g = ($rgba & 0x00FF00) >> 8;
					$b = ($rgba & 0x0000FF);
					$ttrans = imagecolorallocatealpha($reflect, $r, $g, $b, ($rgba >> 24 >= 124 ? 127 : $opacity));

					imagesetpixel($reflect, $x, $y + $h + $spacing, $ttrans);
				
				}
			}
		}
	
		if($hori){
			// horizontal
			for($x = 0; $x < $nw; $x++){
			
				$opacity = $decay + ceil(127 * (($x + $decay) / $nw));
				if($opacity > 127){ $opacity = 127; }
					
				for($y = 0; $y < $nh; $y++){

					$rgba = imagecolorat($im, $w - $x - 1, $y);
					// FIXME: if $rgba >> 24 === 127, we need to muck opacity and keep it invisible, 
					// otherwise our decay makes it visible but slightly opaque?
				
					// break into funciton:
					$red = ($rgba & 0xFF0000) >> 16;
					$green = ($rgba & 0x00FF00) >> 8;
					$blue = ($rgba & 0x0000FF);
					$ttrans = imagecolorallocatealpha($reflect, $red, $green, $blue, ($rgba >> 24 >= 124 ? 127 : $opacity));

					imagesetpixel($reflect, $x + $w + $spacing, $y, $ttrans);
				}
			}
		}
	
		imagesavealpha($reflect, true);
		return $reflect;
	
	}

	function imageSkew($im, $angle, $dir = 0){ 
		// Skews an image handle by some angle either left or right

		$w = imagesx($im);
		$h = imagesy($im);

		$canvas = @imagecreatetruecolor($w, $h); 

		imagealphablending($canvas, false);
		$trans = imagecolorallocatealpha($canvas, 0, 0, 0, 127);
		imagefill($canvas, 0,0, $trans);	 

		// Pixel differences 
		$diff = ($angle / 90); 

		// Loop trough each width pixel 
		$currentHeight = $h; 
		$currentY = 0;
		if($dir == 1){
			$currentHeight = 0; 
			$currentY = $h;
		}
		for($i = 0; $i < $w; $i++){ 
			// Take 1*height sample and copy to iCanvas 
			if($dir == 0){
				imagecopyresampled($canvas, $im, $i, $currentY, $i, 0, 1, $currentHeight, 1, $h); 
				$currentHeight = $currentHeight - ($diff * 2); 
				$currentY = ($h - $currentHeight) / 2;
			}else{
				imagecopyresampled($canvas, $im, ($w - $i), $currentY, ($w - $i), 0, 1, $currentHeight, 1, $h); 
				$currentHeight = $h - ( $i * ($diff * 2) );
				$currentY = ($h - $currentHeight) / 2; 
			}

		} 

		imagesavealpha($canvas, true);
		return $canvas; // Image

	}


?>