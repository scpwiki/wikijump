<?php
// vim: set expandtab tabstop=4 shiftwidth=4 softtabstop=4:
/**
 *
 * @category   Text
 * @package    Text_Wiki
 * @author     Michal Frackowiak
 * @license    http://www.gnu.org/copyleft/lesser.html  LGPL License 2.1
 * @version    $Id$
 * @link       http://pear.php.net/package/Text_Wiki
 */

/**
 * Renders collapsible block.
 *
 * @category   Text
 * @package    Text_Wiki
 * @author     Michal Frackowiak
 * @license    http://www.gnu.org/copyleft/lesser.html  LGPL License 2.1
 * @version    Release: @package_version@
 * @link       http://pear.php.net/package/Text_Wiki
 */
class Text_Wiki_Render_Xhtml_Collapsible extends Text_Wiki_Render {

    public $conf = array();

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
    		$args = $options['args'];
    		if($args['folded'] == 'no' || $args['folded'] == 'false'){
    			$folded = false;
    		}else{
    			$folded = true;
    		}

    		if($args['show']){
    			$show = $args['show'];
    		}else{
    			$show = "+ show block";
    		}

   			if($args['hide']){
    			$hide = $args['hide'];
    		}else{
    			$hide = "- hide block";
    		}

    		if($args['hideLocation'] && $args['hideLocation'] == 'bottom'){
    			$hideLocation = 'bottom';
    		}elseif($args['hideLocation'] && $args['hideLocation'] == 'both'){
    			$hideLocation = 'both';
    		}else{
    			$hideLocation = 'top';
    		}

    		$count = $options['count'];

    		$hideB = '<div><a href="javascript:;" onclick="$(\'collapsible-block-'.$count.'-unfolded\').style.display=\'none\';$(\'collapsible-block-'.$count.'-folded\').style.display=\'block\';">'.htmlspecialchars($hide).'</a></div>';

    	 	if ($options['type'] == 'start') {

    	 		$out  = '<div class="collapsible-block">';
    	 		$out .= '<div id="collapsible-block-'.$count.'-folded" '.((!$folded)?'style="display:none"':'').'>';
    	 		$out .= '<a href="javascript:;" onclick="$(\'collapsible-block-'.$count.'-folded\').style.display=\'none\';$(\'collapsible-block-'.$count.'-unfolded\').style.display=\'block\'; ">'.htmlspecialchars($show).'</a>';
    	 		$out .= '</div>';
    	 		$out .= '<div id="collapsible-block-'.$count.'-unfolded" '.($folded?'style="display:none"':'').'>';
    	 		if($hideLocation == 'top' || $hideLocation == 'both'){
    	 			$out .= $hideB;
    	 		}
    	 		$out .= '<div id="collapsible-block-'.$count.'-content">';

            	return $out;

        }

        if ($options['type'] == 'end') {
            $out =  "</div>";
            if($hideLocation == 'bottom' || $hideLocation == 'both'){
            	$out .= $hideB;
            }
            $out .= "</div></div>";
            return $out;
        }

    }
}
