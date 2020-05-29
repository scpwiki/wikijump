<?php
/**
 * @category   Text
 * @package    Text_Wiki
 * @author     Michal Frackowiak
 * @license    http://www.gnu.org/copyleft/lesser.html  LGPL License 2.1
 * @version    $Id$
 * @link       http://pear.php.net/package/Text_Wiki
 */

/**
 * Gallery.
 *
 * @category   Text
 * @package    Text_Wiki
 * @author     Michal Frackowiak
 * @license    http://www.gnu.org/copyleft/lesser.html  LGPL License 2.1
 * @version    Release: @package_version@
 * @link       http://pear.php.net/package/Text_Wiki
 */
class Text_Wiki_Render_Xhtml_Gallery extends Text_Wiki_Render {

     public $conf = array(
        'base' => '/',
        'url_base' => null,
        'css'  => null,
        'css_link' => null
    );

    /**
    *
    * Renders a token into text matching the requested format.
    *
    * @access public
    *
    * @param array $options The "options" portion of the token (second
    * element).
    *
    * @return string The text rendered from the token options.
    *
    */
 
    function token($options)
    {
    
    	$pageName = $this->wiki->vars['pageName'];	
    		
    	$size = $options['size'];
    			
    	if(!in_array($size, array("small", "medium", "thumbnail", "square", "original"))){
	    		$size = "thumbnail";
	    }
        
        $sources = $options['sources'];
      
      	if($sources){
      		
      		$noLocal = $this->getConf("no_local");
      		
      		// each line is a source + parameters.
      		// parse in a similar way as the image rule.	
      		
      		// parse sources
      		$sources = explode("\n", $sources);
      		if(count($sources) === 0){
      			return '<div class="error-block">'._('Sorry, no images found.').'</div>';
      		}
      		
      		$out = '<div class="gallery-box">';
      		
      		foreach($sources as $row){
      			if(!preg_match("/^: /", $row)){
      				continue;
      			}
      			$row = trim(preg_replace("/^: /", '', $row));
      			$pos = strpos($row, ' ');
      			if ($pos === false) {
      				$src = $row;
      				$attr = array();
      			}else{
      				 // everything after the space is attribute arguments
      				 $src = substr($row, 0, $pos);
      				 $attr = $this->getAttrs(substr($row, $pos+1));	
      			}
      			
      			// SINGLE IMAGE PROCESSING BEGINS

				if($src && $src{0} == '*'){
					$src = substr($src, 1);
				   	$target  = 'target="_blank"';
				}

		      	// see if is a flickr image
		        if (strpos($src, 'flickr:') !== false) {
		        		preg_match("/^flickr:([0-9]+)(?:_([a-z0-9]+))?/i", $src, $mat2);
		        		$photoId = $mat2[1];
		        		$secret = $mat2[2];

		        		$flickr = FlickrHandler::instance();
					$photo = $flickr->photos_getInfo($photoId, $secret);
					
					if($photo == null){
						return '<div class="error-block">Error fetching flickr image (id: '.$photoId.') info. ' .
								'The file does not exist, is private or other problem.</div>';
					}
					
					$src = $flickr->buildPhotoURL($photo, $size); //"http://static.flickr.com/".$photo['_attributes']['server']."/".$photo['_attributes']['id']."_".$photo['_attributes']['secret'].".jpg"; 	
		         	// set/override link attribute
		         	$attr['link'] = $photo['urls']['url'][0]['_value'];
		        }elseif (strpos($src, '://') === false) {
		        		// 	is the source a local file or URL?
		            // the source refers to a local file.
		            // add the URL base to it.

		            // see if it refers to a different page
		            if (strpos($src, '/') !== false) {
		            		$src = preg_replace("/^\//", '', $src);
		            		$osrc = "/local--files/".$src;
            				$src = "/local--resized-images/".$src.'/'.$size.'.jpg';
		            }else{
		            		if($noLocal){
	    						return '<div class="error-block">' .
						    				'Error fetching local image (: '.$row.').' .
						    				'Sorry, can not load files attached to this page in this mode. ' .
						    				'You should specify source page for each local image.</div>';	
						    	}
		            		$osrc = "/local--files/".$this->wiki->vars['pageName'] .'/'. $src;
		            		$src = "/local--resized-images/".
								$this->wiki->vars['pageName'].'/'.$src.'/'.$size.'.jpg';	
		            }
		            if($size == "original"){
		            	$src = $osrc;	
		            }
		        }elseif(strpos($src, '://') !== false){
		        	$attr['link'] = $src;
		        	$size = "original";
		        }else{
		        		return '<div class="error-block">Sorry, format for gallery item:<pre>'.$row.'</pre> is not supported.</div>';	
		        }
		        
		       if (isset($attr['link'])) {
		       		$link = $attr['link'];
		       		if($link{0} == '*'){
	        				$link = substr($link, 1);
	        				$target  = 'target="_blank"';
	        				$attr['link'] = $link;
	     			}
		       	
		            // yes, the image is clickable.
		            // are we linked to a URL or a wiki page?
		            if (strpos($attr['link'], '://')) {
		                // it's a URL
		                $href = $attr['link'];
		            } else {
		                // it's a WikiPage; assume it exists.
		                /** @todo This needs to honor sprintf wikilinks (pmjones) */
		                /** @todo This needs to honor interwiki (pmjones) */
		                /** @todo This needs to honor freelinks (pmjones) */
		                $href = $this->wiki->getRenderConf('xhtml', 'wikilink', 'view_url') .
		                    $attr['link'];
		            }
		        } else {
		            // image is not clickable.
		            $href = $osrc;
		        }

		      	$out .= '<div class="gallery-item '.$size.'">';	
    				$out .= '<table><tr><td>';
    				$out .= '<a href="'.$href.'" '.$target.'>';
    				$out .= '<img src="'.$src.'" alt=""/>';
    				$out .= '</a>';
    				$out .= '</td></tr></table>';
    				$out .= '</div>';
		      			
      			// SINGLE IMAGE PROCESSING ENDS
      			
      		}
            
        		$out .= '</div>';

      		return $out;
      	}
        
        // local mode
        $noLocal = $this->getConf("no_local");
	   	if($noLocal){
	    		return '<div class="error-block">' .
	    				'Sorry, can not load files attached to the page in this mode. ' .
	    				'You should specify sources for each image.</div>';	
	    	}
            	
    		// get page first
    		
    		$site = $GLOBALS['site'];
    		$page = DB_PagePeer::instance()->selectByName($site->getSiteId(), $pageName);
    		if($page == null){
    			return '<div class="error-block">Error selecting page.</div>';	
    		}
    		// get attachments that might be images.
    		$c = new Criteria();
    		$c->add("page_id", $page->getPageId());
    		$c->add("mimetype", "^image", "~*");
    		$c->add("has_resized", true);
    		$files = DB_FilePeer::instance()->select($c);
    		
    		if(count($files) == 0){
    			return '<div class="error-block">Sorry, no images found attached ' .
    					'to this page.</div>';	
    		}
    		
    		// ok, we have images. now GOGOGO!!!
    		$out = '<div class="gallery-box">';
    		
    		foreach($files as $file){
    			
    			$src = '/local--resized-images/'.
					$pageName.'/'.$file->getFilename().'/'.$size.'.jpg';
    			$href = 'local--files/'.$pageName.'/'.$file->getFilename();
    			
    			if($size == "original"){
		            $src = $href;
		         
		        }
    			
    			$out .= '<div class="gallery-item '.$size.'">';	
    			$out .= '<table><tr><td>';
    			$out .= '<a href="'.$href.'">';
    			$out .= '<img src="'.$src.'" alt=""/>';
    			$out .= '</a>';
    			$out .= '</td></tr></table>';
    			$out .= '</div>';
    		}

    		$out .= '</div>';
    			
    	 	return $out;

    }
    
     function getAttrs($text)
    {
        $tmp = explode('="', trim($text));

        // basic setup
        $k = count($tmp) - 1;
        $attrs = array();
        $key = null;

        // loop through the sections
        foreach ($tmp as $i => $val) {

            // first element is always the first key
            if ($i == 0) {
                $key = trim($val);
                continue;
            }

            // find the last double-quote in the value.
            // the part to the left is the value for the last key,
            // the part to the right is the next key name
            $pos = strrpos($val, '"');
            $attrs[$key] = stripslashes(substr($val, 0, $pos));
            $key = trim(substr($val, $pos+1));

        }

        return $attrs;

    }
}
