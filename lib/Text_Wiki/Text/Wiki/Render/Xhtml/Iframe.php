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
 * Iframe element.
 *
 * @category   Text
 * @package    Text_Wiki
 * @author     Michal Frackowiak
 * @license    http://www.gnu.org/copyleft/lesser.html  LGPL License 2.1
 * @version    Release: @package_version@
 * @link       http://pear.php.net/package/Text_Wiki
 */
class Text_Wiki_Render_Xhtml_Iframe extends Text_Wiki_Render {

    function token($options){

       	$output = '<iframe ';
       	foreach($options as $k => $o){
       		$output .= $k.'="'.htmlspecialchars($o).'" ';
       	}
       	$output .='></iframe>';

        return $output;
    }
}
