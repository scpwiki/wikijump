<?php
// vim: set expandtab tabstop=4 shiftwidth=4 softtabstop=4:
/**
 * Image rule end renderer for Xhtml
 *
 * PHP versions 4 and 5
 *
 * @category   Text
 * @package    Text_Wiki
 * @author     Paul M. Jones <pmjones@php.net>
 * @author Michal Frackowiak
 * @license    http://www.gnu.org/copyleft/lesser.html  LGPL License 2.1
 * @version    $Id$
 * @link       http://pear.php.net/package/Text_Wiki
 */

/**
 * This class inserts an image in XHTML.
 *
 * @category   Text
 * @package    Text_Wiki
 * @author     Paul M. Jones <pmjones@php.net>
 * @author Michal Frackowiak
 * @license    http://www.gnu.org/copyleft/lesser.html  LGPL License 2.1
 * @version    Release: @package_version@
 * @link       http://pear.php.net/package/Text_Wiki
 */
class Text_Wiki_Render_Xhtml_Image extends Text_Wiki_Render {

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
        // note the image source
        $src = $options['src'];

        $size = $options['attr']['size'];
        
        $postVars = $this->getConf("post_vars");
        
        if (preg_match('/^:first/', $src)) {
    		$page = DB_PagePeer::instance()->selectByName($GLOBALS['site']->getSiteId(), $this->wiki->vars['pageName']);
    		if (! $page) {
    			return "";
    		}
    		$c = new Criteria();
    		$c->add("page_id", $page->getPageId());
    		$c->add("mimetype", "^image", "~*");
    		$c->add("has_resized", true);
    		$c->addOrderAscending("filename");
    		if ($file = DB_FilePeer::instance()->selectOne($c)) {
    			$src = $file->getFileName();
    			$options['src'] = $src;
    		} else {
    			return "";
    		}
        }
        
        // see if is a flickr image
        if (strpos($src, 'flickr:') !== false) {
        		//check if valid arguments, handle sizes etc. 
        		preg_match("/^flickr:([0-9]+)(?:_([a-z0-9]+))?/i", $src, $mat2);
        		$photoId = $mat2[1];
        		$secret = $mat2[2];
        		if(!in_array($size, array("small", "medium", "thumbnail", "square", "large", "original"))){
	        		$size = null;	
	        }
        		
        		$flickr = FlickrHandler::instance();
			$photo = $flickr->photos_getInfo($photoId, $secret);
			
			if($photo == null){
				return '<div class="error-block">'.sprintf(_('Error fetching flickr image (id: %s) info. The file does not exist, is private or other problem.'),$photoId).'</div>';
			}
			
			$src = $flickr->buildPhotoURL($photo, $size); //"http://static.flickr.com/".$photo['_attributes']['server']."/".$photo['_attributes']['id']."_".$photo['_attributes']['secret'].".jpg"; 	
         	// set/override link attribute
         	$options['attr']['link'] = $photo['urls']['url'][0]['_value'];
        }elseif (strpos($src, '://') === false) {
        		// 	is the source a local file or URL?
            // the source refers to a local file.
            // add the URL base to it.
            if(!in_array($size, array("small", "medium", "thumbnail", "square"))){
	        		$size = null;	
	        }
	        
	        // should we force size? yes if the file has resized version but is 
	        // not a viewable image, e.g. .pdf
	        
	        if(!$size && !preg_match('/\.(png|jpg|gif)$/i', $src)){
	        		$size = "medium";	
	        }
            
            // see if it refers to a different page
            	if($postVars || preg_match("/^%%%[^%]+%%$/", $src)){
            			// this is ok. used for feed parsing.
            	}elseif (strpos($src, '/') !== false) {
            		if($size){
            			$src = preg_replace("/^\//", '', $src);
            			$src = "/local--resized-images/".$src.'/'.$size.'.jpg';
            			if($options['attr']['link'] == null){
            				// link to the original image
            				$srch = preg_replace("/^\//", '', $options['src']);
            				$options['attr']['link'] = 	"/local--files/".$srch;
            			}
            		}else{
	            		// 	ok, hardcode the path... sorry.
            			$src = preg_replace("/^\//", '', $src);
            			$src = "/local--files/".$src;
            		}
            }else{

		        	$noLocal = $this->getConf("no_local");
		    		if($noLocal){
		    			return '<span class="error-inline">' .
		   					_('Sorry, local images without page name specified not allowed. Use <em>pagename</em>/<em>filename</em> as the image source').'</span>';	
		    		}
		      		
	       		if($size){
	        			$src = "/local--resized-images/".
					$this->wiki->vars['pageName'].'/'.$src.'/'.$size.'.jpg';
					if($options['attr']['link'] == null){
            				// link to the original image
            				$srch = preg_replace("/^\//", '', $options['src']);
            				$options['attr']['link'] = 	"/local--files/".$this->wiki->vars['pageName'] .'/'. $srch;
            			}	
	            	}else{
	         		$src = "/local--files/".$this->wiki->vars['pageName'] .'/'. $src;
	         	}
            }
        }

        if (isset($options['attr']['link'])) {
            // yes, the image is clickable.
            // are we linked to a URL or a wiki page?
            if (strpos($options['attr']['link'], '://')) {
                // it's a URL, prefix the URL base
                $href = $this->getConf('url_base') . $options['attr']['link'];
            } else {
                // it's a WikiPage; assume it exists.
                /** @todo This needs to honor sprintf wikilinks (pmjones) */
                /** @todo This needs to honor interwiki (pmjones) */
                /** @todo This needs to honor freelinks (pmjones) */
                $href = '/'.$options['attr']['link'];
                $href = preg_replace(';^/+;', '/',$href); // a dirty fix
            }
        } else {
            // image is not clickable.
            $href = null;
        }
        // unset so it won't show up as an attribute
        unset($options['attr']['link']);

        $output .= '<img src="' . htmlspecialchars($src) . '"';

        // get the CSS class but don't add it yet
        $css = "image";

        // add the attributes to the output, and be sure to
        // track whether or not we find an "alt" attribute
        $alt = false;
        foreach ($options['attr'] as $key => $val) {

            // track the 'alt' attribute
            if (strtolower($key) == 'alt') {
                $alt = true;
            }

            // the 'class' attribute overrides the CSS class conf
            if (strtolower($key) == 'class') {
                $css = null;
            }

			if($key == "class" || $key == "alt"  || $key == "style" 
				|| $key=="width" || $key=="height"){          
	            	$key = htmlspecialchars($key);
            		$val = htmlspecialchars($val);
            		$output .= " $key=\"$val\"";
			}
        }

        // always add an "alt" attribute per Stephane Solliec
        if (! $alt) {
            $alt = htmlspecialchars(basename($options['src']));
            $output .= " alt=\"$alt\"";
        }

        // end the image tag with the automatic CSS class (if any)
        if($css){
        		$output .= ' class="'.$css.'"';
        }
        $output .= " />";

        // was the image clickable?
        if ($href) {
            // yes, add the href and return
            $href = htmlspecialchars($href);
            $css = $this->formatConf(' class="%s"', 'css_link');
            $target = '';
            if($options['target']){
            		$target = ' target="'.$options['target'].'" ';	
            }
            $output = "<a$css href=\"$href\" $target>$output</a>";
        }
        
        $align = $options['align'];
		
		if($align){
			
			$output2 = $output;
			$output = '';
			
	        // start the HTML output
	        $output .= '<div class="image-container';
	        
	        if($align === "f<"){
				$output .= ' floatleft';
			}elseif($align ==="f>"){
				$output .= ' floatright';
			}elseif($align === "<"){
				$output .= ' alignleft';
			}elseif($align === ">"){
				$output .= ' alignright';
			}elseif($align === '='){
				$output .= ' aligncenter';
	        }
	        
			$output .= '">';
			
			$output .= $output2;
      		$output .= '</div>';
      	}

        return $output;
    }
}
